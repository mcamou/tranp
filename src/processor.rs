use std::collections::HashMap;

use crate::types::Account::{Locked, Unlocked};
use crate::types::{Account, AccountData, ClientId, Error, Txn, TxnId};

#[derive(Debug)]
pub struct Processor {
    accounts: HashMap<ClientId, Account>,
    history: HashMap<(TxnId, ClientId), Txn>,
    disputes: HashMap<(TxnId, ClientId), Txn>,
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            accounts: HashMap::new(),
            history: HashMap::new(),
            disputes: HashMap::new(),
        }
    }

    pub fn process_txn(&mut self, txn: &Txn) -> Result<(), Error> {
        match txn {
            Txn::Deposit { client, tx, amount } => match self.accounts.get_mut(client) {
                Some(Unlocked(acct)) => {
                    acct.available = acct.available + (*amount);
                    self.history.insert((*tx, *client), (*txn).clone());
                    Ok(())
                }

                None => {
                    let ac = Unlocked(AccountData {
                        client: *client,
                        available: *amount,
                        held: 0.into(),
                    });
                    self.accounts.insert(*client, ac);
                    self.history.insert((*tx, *client), (*txn).clone());
                    Ok(())
                }
                _ => Err(Error::LockedAccount(*tx, *client)),
            },

            Txn::Withdrawal { client, tx, amount } => match self.accounts.get_mut(client) {
                Some(Unlocked(acct)) if *amount <= acct.available => {
                    acct.available = acct.available - (*amount);
                    self.history.insert((*tx, *client), txn.clone());
                    Ok(())
                }

                Some(Unlocked(..)) => Err(Error::InsufficientFunds(*tx, "withdrawal".to_string())),

                Some(Locked(..)) => Err(Error::LockedAccount(*tx, *client)),

                None => Err(Error::NonexistentAccount(*tx, *client)),
            },

            Txn::Dispute { client, tx } => match self.accounts.get_mut(client) {
                Some(Unlocked(acct)) => match self.history.get(&(*tx, *client)) {
                    Some(
                        t @ Txn::Deposit { tx, amount, .. }
                        | t @ Txn::Withdrawal { tx, amount, .. },
                    ) if *amount <= acct.available => {
                        acct.available = acct.available - (*amount);
                        acct.held = acct.held + (*amount);
                        self.disputes.insert((*tx, *client), (*t).clone());
                        Ok(())
                    }

                    Some(Txn::Deposit { tx, .. } | Txn::Withdrawal { tx, .. }) => {
                        Err(Error::InsufficientFunds(*tx, "dispute".to_string()))
                    }

                    _ => Err(Error::InvalidTransaction(
                        *tx,
                        "Invalid dispute".to_string(),
                    )),
                },

                Some(Locked(..)) => Err(Error::LockedAccount(*tx, *client)),

                None => Err(Error::NonexistentAccount(*tx, *client)),
            },

            Txn::Resolve { client, tx } => match self.accounts.get_mut(client) {
                Some(Unlocked(acct)) => match self.disputes.get(&(*tx, *client)) {
                    Some(Txn::Deposit { tx, amount, .. } | Txn::Withdrawal { tx, amount, .. })
                        if *amount <= acct.held =>
                    {
                        acct.available = acct.available + (*amount);
                        acct.held = acct.held - (*amount);
                        self.disputes.remove(&(*tx, *client));
                        Ok(())
                    }

                    Some(Txn::Deposit { tx, .. } | Txn::Withdrawal { tx, .. }) => {
                        Err(Error::InsufficientFunds(*tx, "resolve".to_string()))
                    }

                    _ => Err(Error::InvalidTransaction(
                        *tx,
                        "Invalid resolve".to_string(),
                    )),
                },

                Some(Locked(..)) => Err(Error::LockedAccount(*tx, *client)),

                None => Err(Error::NonexistentAccount(*tx, *client)),
            },

            Txn::Chargeback { client, tx } => match self.accounts.get_mut(client) {
                Some(Unlocked(acct)) => match self.history.get(&(*tx, *client)) {
                    Some(Txn::Deposit { tx, amount, .. } | Txn::Withdrawal { tx, amount, .. })
                        if *amount <= acct.held =>
                    {
                        let ac = Locked(AccountData {
                            client: *client,
                            available: acct.available,
                            held: acct.held - (*amount),
                        });
                        self.accounts.insert(*client, ac);
                        self.disputes.remove(&(*tx, *client));
                        Ok(())
                    }

                    Some(Txn::Deposit { tx, .. } | Txn::Withdrawal { tx, .. }) => {
                        Err(Error::InsufficientFunds(*tx, "chargeback".to_string()))
                    }

                    _ => Err(Error::InvalidTransaction(
                        *tx,
                        "Invalid chargeback".to_string(),
                    )),
                },

                Some(Locked(_)) => Err(Error::LockedAccount(*tx, *client)),

                None => Err(Error::NonexistentAccount(*tx, *client)),
            },
        }
    }

    pub fn get_accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposit_creates_account() {
        let mut p = Processor::new();
        let txn = Txn::Deposit {
            client: 42,
            tx: 4242,
            amount: 42.into(),
        };

        let _ = p.process_txn(&txn);

        let expected = Unlocked(AccountData {
            client: 42,
            available: 42.into(),
            held: 0.into(),
        });

        let acct = p.accounts.get(&42).cloned().expect("Account not found");
        assert_eq!(acct, expected);

        let hist_txn = p
            .history
            .get(&(4242, 42))
            .cloned()
            .expect("Transaction not found");
        assert_eq!(hist_txn, txn);
    }

    #[test]
    fn other_txn_error_if_no_acct() {
        let mut p = Processor::new();

        let txn = Txn::Withdrawal {
            client: 42,
            tx: 4242,
            amount: 42.into(),
        };
        let actual = p.process_txn(&txn);
        let expected = Err(Error::NonexistentAccount(4242, 42));
        assert_eq!(actual, expected);

        let txn = Txn::Dispute {
            client: 42,
            tx: 4242,
        };
        let actual = p.process_txn(&txn);
        assert_eq!(actual, expected);

        let txn = Txn::Resolve {
            client: 42,
            tx: 4242,
        };
        let actual = p.process_txn(&txn);
        assert_eq!(actual, expected);

        let txn = Txn::Chargeback {
            client: 42,
            tx: 4242,
        };
        let actual = p.process_txn(&txn);
        assert_eq!(actual, expected);
    }

    #[test]
    fn deposit_and_withdrawal() {
        let mut p = Processor::new();

        let txs = vec![
            (
                Txn::Deposit {
                    client: 42,
                    tx: 4242,
                    amount: 4242.into(),
                },
                Unlocked(AccountData {
                    client: 42,
                    available: 4242.into(),
                    held: 0.into(),
                }),
            ),
            (
                Txn::Withdrawal {
                    client: 42,
                    tx: 4243,
                    amount: 42.into(),
                },
                Unlocked(AccountData {
                    client: 42,
                    available: 4200.into(),
                    held: 0.into(),
                }),
            ),
        ];

        for (txn, expected_acct) in txs {
            let result = p.process_txn(&txn);
            assert_eq!(result, Ok(()));
            let acct = p.accounts.get(&42).cloned().expect("Account not found");
            assert_eq!(acct, expected_acct);
        }
    }

    #[test]
    fn chargeback_locks_account() {
        let mut p = Processor::new();

        let txs = vec![
            Txn::Deposit {
                client: 42,
                tx: 4242,
                amount: 42.into(),
            },
            Txn::Dispute {
                client: 42,
                tx: 4242,
            },
            Txn::Chargeback {
                client: 42,
                tx: 4242,
            },
        ];

        for txn in txs {
            let result = p.process_txn(&txn);
            assert_eq!(result, Ok(()));
        }

        let expected = Locked(AccountData {
            client: 42,
            available: 0.into(),
            held: 0.into(),
        });
        let actual = p.accounts.get(&42).cloned().expect("Account not found");
        assert_eq!(actual, expected);
    }

    #[test]
    fn dispute_resolution() {
        let mut p = Processor::new();

        let txs = vec![
            (
                Txn::Deposit {
                    client: 42,
                    tx: 4242,
                    amount: 42.into(),
                },
                Unlocked(AccountData {
                    client: 42,
                    available: 42.into(),
                    held: 0.into(),
                }),
            ),
            (
                Txn::Dispute {
                    client: 42,
                    tx: 4242,
                },
                Unlocked(AccountData {
                    client: 42,
                    available: 0.into(),
                    held: 42.into(),
                }),
            ),
            (
                Txn::Resolve {
                    client: 42,
                    tx: 4242,
                },
                Unlocked(AccountData {
                    client: 42,
                    available: 42.into(),
                    held: 0.into(),
                }),
            ),
        ];

        for (txn, expected_acct) in txs {
            let result = p.process_txn(&txn);
            assert_eq!(result, Ok(()));
            let acct = p.accounts.get(&42).cloned().expect("Account not found");
            assert_eq!(acct, expected_acct);
        }
    }

    #[test]
    fn ignore_nonexistent_dispute() {
        let mut p = Processor::new();

        let txn = Txn::Deposit {
            client: 42,
            tx: 42,
            amount: 42.into(),
        };
        let _ = p.process_txn(&txn);

        let txn = Txn::Dispute {
            client: 42,
            tx: 4242,
        };
        let actual = p.process_txn(&txn);

        let expected = Err(Error::InvalidTransaction(
            4242,
            "Invalid dispute".to_string(),
        ));
        assert_eq!(actual, expected);
    }

    #[test]
    fn ignore_nonexistent_resolve() {
        let mut p = Processor::new();

        let txn = Txn::Deposit {
            client: 42,
            tx: 42,
            amount: 42.into(),
        };
        let _ = p.process_txn(&txn);

        let txn = Txn::Resolve {
            client: 42,
            tx: 4242,
        };
        let actual = p.process_txn(&txn);

        let expected = Err(Error::InvalidTransaction(
            4242,
            "Invalid resolve".to_string(),
        ));
        assert_eq!(actual, expected);
    }

    #[test]
    fn ignore_nonexistent_chargeback() {
        let mut p = Processor::new();

        let txn = Txn::Deposit {
            client: 42,
            tx: 42,
            amount: 42.into(),
        };
        let _ = p.process_txn(&txn);

        let txn = Txn::Chargeback {
            client: 42,
            tx: 4242,
        };
        let actual = p.process_txn(&txn);

        let expected = Err(Error::InvalidTransaction(
            4242,
            "Invalid chargeback".to_string(),
        ));
        assert_eq!(actual, expected);
    }

    #[test]
    fn locked_accounts() {
        let mut p = Processor::new();

        // Set the account to a Locked state
        let txn = Txn::Deposit {
            client: 42,
            tx: 4242,
            amount: 42.into(),
        };
        let _ = p.process_txn(&txn);
        let txn = Txn::Dispute {
            client: 42,
            tx: 4242,
        };
        let _ = p.process_txn(&txn);
        let txn = Txn::Chargeback {
            client: 42,
            tx: 4242,
        };
        let _ = p.process_txn(&txn);

        let actual_acct = p.accounts.get(&42).cloned().expect("Account not found");
        let expected_acct = Locked(AccountData {
            client: 42,
            available: 0.into(),
            held: 0.into(),
        });
        assert_eq!(actual_acct, expected_acct);

        // And now let's test
        let expected = Err(Error::LockedAccount(4243, 42));

        let txn = Txn::Deposit {
            client: 42,
            tx: 4243,
            amount: 42.into(),
        };
        let actual = p.process_txn(&txn);
        assert_eq!(actual, expected);

        let txn = Txn::Withdrawal {
            client: 42,
            tx: 4243,
            amount: 42.into(),
        };
        let actual = p.process_txn(&txn);
        assert_eq!(actual, expected);

        let txn = Txn::Dispute {
            client: 42,
            tx: 4243,
        };
        let actual = p.process_txn(&txn);
        assert_eq!(actual, expected);

        let txn = Txn::Resolve {
            client: 42,
            tx: 4243,
        };
        let actual = p.process_txn(&txn);
        assert_eq!(actual, expected);

        let txn = Txn::Chargeback {
            client: 42,
            tx: 4243,
        };
        let actual = p.process_txn(&txn);
        assert_eq!(actual, expected);
    }
}

use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::types::{Account, ClientId, Error, Txn, TxnId};

// The csv crate does not support internally-tagged unions: https://github.com/BurntSushi/rust-csv/issues/211
#[derive(Deserialize, Debug)]
pub struct Input {
    #[serde(rename = "type")]
    tpe: String,
    client: ClientId,
    tx: TxnId,
    amount: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Output {
    pub client: ClientId,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}

impl TryFrom<Input> for Txn {
    type Error = Error;

    fn try_from(inp: Input) -> Result<Txn, Self::Error> {
        match inp.tpe.as_str() {
            "deposit" => match inp.amount {
                Some(amt) => amt.try_into().map(|a| Txn::Deposit {
                    client: inp.client,
                    tx: inp.tx,
                    amount: a,
                }),
                None => Err(Error::Input(format!(
                    "Missing amount in transaction {}",
                    inp.tx
                ))),
            },
            "withdrawal" => match inp.amount {
                Some(amt) => amt.try_into().map(|a| Txn::Withdrawal {
                    client: inp.client,
                    tx: inp.tx,
                    amount: a,
                }),
                None => Err(Error::Input(format!(
                    "Missing amount in transaction {}",
                    inp.tx
                ))),
            },
            "dispute" => Ok(Txn::Dispute {
                client: inp.client,
                tx: inp.tx,
            }),
            "resolve" => Ok(Txn::Resolve {
                client: inp.client,
                tx: inp.tx,
            }),
            "chargeback" => Ok(Txn::Chargeback {
                client: inp.client,
                tx: inp.tx,
            }),
            _ => Err(Error::Input(format!(
                "Invalid transaction type in transaction {}",
                inp.tx
            ))),
        }
    }
}

impl From<&Account> for Output {
    fn from(val: &Account) -> Self {
        match val {
            Account::Locked(a) => Output {
                client: a.client,
                available: (&a.available).into(),
                held: (&a.held).into(),
                total: (&(a.available + a.held)).into(),
                locked: true,
            },
            Account::Unlocked(a) => Output {
                client: a.client,
                available: (&a.available).into(),
                held: (&a.held).into(),
                total: (&(a.available + a.held)).into(),
                locked: false,
            },
        }
    }
}

pub fn save<'a, I: Iterator<Item = &'a Account>>(
    writer: impl Write,
    accts: I,
) -> Result<(), Error> {
    let out = accts.map(|a| -> Output { a.into() });
    let mut wrt = csv::Writer::from_writer(writer);
    for o in out {
        if let Err(e) = wrt.serialize(o) {
            return Err(Error::Serialization(e.to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::BufWriter;

    use crate::types::AccountData;

    use super::*;

    #[test]
    fn test_deserialize_transaction() {
        let csv_str = r#"
type,client,tx,amount
deposit,1,2,3
withdrawal,1,2,3.5
dispute,1,2,
resolve,1,2
chargeback,1,2"#;

        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(csv_str.as_bytes());
        let actual: Vec<Txn> = rdr
            .deserialize::<Input>()
            .map(|x| x.unwrap().try_into().unwrap())
            .collect();
        let expected = vec![
            Txn::Deposit {
                client: 1,
                tx: 2,
                amount: 30000.into(),
            },
            Txn::Withdrawal {
                client: 1,
                tx: 2,
                amount: 35000.into(),
            },
            Txn::Dispute { client: 1, tx: 2 },
            Txn::Resolve { client: 1, tx: 2 },
            Txn::Chargeback { client: 1, tx: 2 },
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_accounts() {
        let accts: Vec<Output> = vec![
            (&Account::Unlocked(AccountData {
                client: 1,
                available: 30000.into(),
                held: 40000.into(),
            }))
                .into(),
            (&Account::Locked(AccountData {
                client: 2,
                available: 31111.into(),
                held: 42222.into(),
            }))
                .into(),
        ];

        let buf = BufWriter::new(Vec::new());
        let mut wrt = csv::Writer::from_writer(buf);
        for acct in accts {
            wrt.serialize(acct).expect("Cannot serialize");
        }
        wrt.flush().expect("Cannot flush");
        let bytes = wrt.into_inner().expect("").into_inner().expect("");
        let actual = String::from_utf8(bytes).expect("Invalid utf8");

        let expected = r#"client,available,held,total,locked
1,3.0000,4.0000,7.0000,false
2,3.1111,4.2222,7.3333,true
"#;
        assert_eq!(actual, expected);
    }
}

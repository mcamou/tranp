use crate::amount::Amount;

pub type ClientId = u16;
pub type TxnId = u32;

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Txn {
    Deposit {
        client: ClientId,
        tx: TxnId,
        amount: Amount,
    },
    Withdrawal {
        client: ClientId,
        tx: TxnId,
        amount: Amount,
    },
    Dispute {
        client: ClientId,
        tx: TxnId,
    },
    Resolve {
        client: ClientId,
        tx: TxnId,
    },
    Chargeback {
        client: ClientId,
        tx: TxnId,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct AccountData {
    pub client: ClientId,
    pub available: Amount,
    pub held: Amount,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Account {
    Locked(AccountData),
    Unlocked(AccountData),
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Error in input data: `{0}`.")]
    Input(String),
    #[error("Insufficient funds ({1}) referencing transaction {1}")]
    InsufficientFunds(TxnId, String),
    #[error("Invalid Transaction {0}: `{1}`")]
    InvalidTransaction(TxnId, String),
    #[error("Transaction {0}: Nonexistent account: {1}")]
    NonexistentAccount(TxnId, ClientId),
    #[error("Transaction {0}: Locked account: {1}")]
    LockedAccount(TxnId, ClientId),
}

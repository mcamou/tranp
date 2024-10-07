use thiserror;

use crate::amount::Amount;

pub type ClientId = u16;
pub type TxnId = u32;

#[derive(Debug, PartialEq, Hash)]
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

pub struct AccountData {
    pub client: ClientId,
    pub available: Amount,
    pub held: Amount,
}

pub enum Account {
    Locked(AccountData),
    Unlocked(AccountData),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error in input data: `{0}`.")]
    Input(String),
}

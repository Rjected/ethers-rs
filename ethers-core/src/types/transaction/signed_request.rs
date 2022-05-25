use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::Signature;

use super::eip2718::TypedTransaction;

/// A typed transaction request
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct SignedTransactionRequest {
    pub tx: TypedTransaction,
    pub sig: Signature,
}

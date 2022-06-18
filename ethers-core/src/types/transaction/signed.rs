use fastrlp::{Encodable, Decodable};
use serde::{Deserialize, Serialize};
use crate::types::Signature;
use super::eip2718::TypedTransaction;

/// Signed tranaction requests are represented by the SignedTransactionRequest struct.
///
/// A [`Signature`] and [`TypedTransaction`] can
/// To support Kovan and other non-London-compatbile networks, please enable
/// the `legacy` crate feature. This will disable the `type` flag in the
/// serialized transaction, and cause contract calls and other common actions
/// to default to using the legacy transaction type.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[cfg_attr(not(feature = "legacy"), serde(tag = "type"))]
pub struct SignedTransactionRequest {
    tx: TypedTransaction,
    sig: Signature,
}

impl Encodable for SignedTransactionRequest {
    fn length(&self) -> usize {
        todo!()
    }
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        todo!()
    }
}

impl Decodable for SignedTransactionRequest {
    fn decode(buf: &mut &[u8]) -> Result<Self, fastrlp::DecodeError> {
        todo!()
    }
}

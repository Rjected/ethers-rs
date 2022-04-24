use rlp::{Encodable, Decodable};
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

// impl Encodable for SignedTransactionRequest {
//     fn rlp_bytes(&self) -> bytes::BytesMut {

//     }
//     fn rlp_append(&self, s: &mut rlp::RlpStream) {

//     }
// }

// impl Decodable for SignedTransactionRequest {
//     fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {

//     }
// }

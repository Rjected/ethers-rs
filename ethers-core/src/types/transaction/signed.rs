use ethabi::ethereum_types::U256;
use fastrlp::{Encodable, Decodable, length_of_length, Header};
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

impl SignedTransactionRequest {
    /// Returns the rlp length of the signed transaction, not including the rlp list header.
    /// To get the length including the rlp list header, refer to the Encodable implementation.
    pub(crate) fn signed_tx_payload_length(&self) -> usize {
        // add each of the fields' rlp encoded lengths
        let mut length = 0;
        length += match &self.tx {
            TypedTransaction::Eip1559(tx) => {
                // it says including but really
                tx.tx_payload_length()
            }
            TypedTransaction::Eip2930(tx) => {
                tx.payload_length()
            }
            TypedTransaction::Legacy(tx) => {
                tx.tx_body_length()
            }
        };

        // the max value for a single byte to represent itself is 0x7f
        let max_for_header = U256::from(0x7fu8);
        // the number of rlp string headers - each U256 can be either a single byte (and is < 0x7f)
        // or less than 32
        let mut headers_len = 0;
        length += self.sig.v.length();

        headers_len += if self.sig.r < max_for_header { 0 } else { 1 };
        length += 32 - self.sig.r.leading_zeros() as usize / 8;

        headers_len += if self.sig.s < max_for_header { 0 } else { 1 };
        length += 32 - self.sig.s.leading_zeros() as usize / 8;

        length += headers_len;
        length
    }
}

impl Encodable for SignedTransactionRequest {
    fn length(&self) -> usize {
        let mut length = 0;
        length += self.signed_tx_payload_length();

        // header would encode length_of_length + 1 bytes
        length += if length > 55 { 1 + length_of_length(length) } else { 1 };
        length
    }
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        let list_header = Header { list: true, payload_length: self.signed_tx_payload_length() };
        list_header.encode(out);

        let mut uint_container = [0x00; 32];
        // TODO: need to make sure every tx type has encode_tx_body and the same name for all
        // methods
        todo!()
    }
}

impl Decodable for SignedTransactionRequest {
    fn decode(buf: &mut &[u8]) -> Result<Self, fastrlp::DecodeError> {
        todo!()
    }
}

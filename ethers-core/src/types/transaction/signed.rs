use ethabi::ethereum_types::U256;
use fastrlp::{Encodable, Decodable, length_of_length, Header};
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use crate::types::{Signature, TransactionRequest, Eip1559TransactionRequest, Eip2930TransactionRequest};
use super::eip2718::TypedTransaction;

/// Signed tranaction requests are represented by the SignedTransactionRequest struct.
///
/// A [`Signature`] and [`TypedTransaction`] can
/// To support Kovan and other non-London-compatbile networks, please enable
/// the `legacy` crate feature. This will disable the `type` flag in the
/// serialized transaction, and cause contract calls and other common actions
/// to default to using the legacy transaction type.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SignedTransactionRequest {
    pub tx: TypedTransaction,
    pub sig: Signature,
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
                tx.tx_body_length()
            }
            TypedTransaction::Eip2930(tx) => {
                tx.tx_body_length()
            }
            TypedTransaction::Legacy(tx) => {
                tx.tx_body_length()
            }
        };

        // the max value for a single byte to represent itself is 0x7f
        let max_for_header = U256::from(fastrlp::EMPTY_STRING_CODE);
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

        match &self.tx {
            TypedTransaction::Eip1559(tx) => {
                tx.encode_tx_body(out)
            }
            TypedTransaction::Eip2930(tx) => {
                tx.encode_tx_body(out)
            }
            TypedTransaction::Legacy(tx) => {
                tx.encode_tx_body(out)
            }
        }

        self.sig.v.encode(out);
        let mut uint_container = [0x00; 32];

        self.sig.r.to_big_endian(&mut uint_container);
        let r_bytes = &uint_container[self.sig.r.leading_zeros() as usize / 8..];
        r_bytes.encode(out);

        self.sig.s.to_big_endian(&mut uint_container);
        let s_bytes = &uint_container[self.sig.s.leading_zeros() as usize / 8..];
        s_bytes.encode(out);
    }
}

impl Decodable for SignedTransactionRequest {
    fn decode(buf: &mut &[u8]) -> Result<Self, fastrlp::DecodeError> {
        let _header = Header::decode(buf)?;

        let tx_type = buf.first();
        let tx = match tx_type {
            Some(&x) if x == 0x01u8 => {
                // EIP-2930 (0x01)
                let request = Eip2930TransactionRequest::decode_tx_body(buf)?;
                TypedTransaction::Eip2930(request)
            }
            Some(&x) if x == 0x02u8 => {
                // EIP-1559 (0x02)
                let request = Eip1559TransactionRequest::decode_tx_body(buf)?;
                TypedTransaction::Eip1559(request)
            }
            _ => {
                // Legacy (0x00)
                // use the original rlp
                let request = TransactionRequest::decode_tx_body(buf)?;
                TypedTransaction::Legacy(request)
            }
        };

        let sig = Signature::decode_signature(buf)?;

        Ok(Self {
            tx,
            sig,
        })
    }
}

// TODO: revise decode impls and comments
// TODO: tests
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ethabi::ethereum_types::H256;

    use crate::types::SignedTransactionRequest;

    #[test]
    fn test_typed_tx_without_access_list() {
        let tx: SignedTransactionRequest = serde_json::from_str(
            r#"{
            "gas": "0x186a0",
            "maxFeePerGas": "0x77359400",
            "maxPriorityFeePerGas": "0x77359400",
            "data": "0x5544",
            "nonce": "0x2",
            "to": "0x96216849c49358B10257cb55b28eA603c874b05E",
            "value": "0x5af3107a4000",
            "type": "0x2",
            "chainId": "0x539",
            "accessList": [],
            "v": "0x1",
            "r": "0xc3000cd391f991169ebfd5d3b9e93c89d31a61c998a21b07a11dc6b9d66f8a8e",
            "s": "0x22cfe8424b2fbd78b16c9911da1be2349027b0a3c40adf4b6459222323773f74"
        }"#,
        )
        .unwrap();

        let expected =
            H256::from_str("0xa1ea3121940930f7e7b54506d80717f14c5163807951624c36354202a8bffda6")
                .unwrap();
        let actual = tx.tx.sighash();
        assert_eq!(expected, actual);
    }
}

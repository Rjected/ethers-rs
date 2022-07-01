use bytes::Buf;
// Adapted from https://github.com/tomusdrw/rust-web3/blob/master/src/types/log.rs
use crate::types::{Address, Bytes, H256, U256, U64};
use fastrlp::{length_of_length, Decodable, Encodable, Header};
use serde::{Deserialize, Serialize};

/// A log produced by a transaction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Log {
    /// H160. the contract that emitted the log
    pub address: Address,

    /// topics: Array of 0 to 4 32 Bytes of indexed log arguments.
    /// (In solidity: The first topic is the hash of the signature of the event
    /// (e.g. `Deposit(address,bytes32,uint256)`), except you declared the event
    /// with the anonymous specifier.)
    pub topics: Vec<H256>,

    /// Data
    pub data: Bytes,

    /// Block Hash
    #[serde(rename = "blockHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<H256>,

    /// Block Number
    #[serde(rename = "blockNumber")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<U64>,

    /// Transaction Hash
    #[serde(rename = "transactionHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<H256>,

    /// Transaction Index
    #[serde(rename = "transactionIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_index: Option<U64>,

    /// Integer of the log index position in the block. None if it's a pending log.
    #[serde(rename = "logIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_index: Option<U256>,

    /// Integer of the transactions index position log was created from.
    /// None when it's a pending log.
    #[serde(rename = "transactionLogIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_log_index: Option<U256>,

    /// Log Type
    #[serde(rename = "logType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_type: Option<String>,

    /// True when the log was removed, due to a chain reorganization.
    /// false if it's a valid log.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<bool>,
}

impl rlp::Encodable for Log {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(3);
        s.append(&self.address);
        s.append_list(&self.topics);
        s.append(&self.data.0);
    }
}

// TODO: Implement more common types - or adjust this to work with all Tokenizable items
impl Log {
    /// Returns the rlp length of the Log body, _including_ trailing EIP155 fields,
    /// but not including the rlp list header.
    /// To get the length including the rlp list header, refer to the Encodable implementation.
    pub(crate) fn log_payload_length(&self) -> usize {
        let mut length = 0;
        length += if self.address == Address::zero() { 1 } else { self.address.length() };
        length += self.topics.length();
        length += self.data.0.length();
        length
    }
}

impl Encodable for Log {
    fn length(&self) -> usize {
        // add each of the fields' rlp encoded lengths
        let mut length = 0;
        length += self.log_payload_length();
        length += length_of_length(length);

        length
    }

    fn encode(&self, out: &mut dyn bytes::BufMut) {
        // [contract-address, topics, data]
        let list_header = Header { list: true, payload_length: self.log_payload_length() };
        list_header.encode(out);

        if self.address == Address::zero() {
            out.put_u8(0x80);
        } else {
            self.address.encode(out);
        }

        self.topics.encode(out);
        self.data.0.encode(out);
    }
}

impl Decodable for Log {
    fn decode(buf: &mut &[u8]) -> Result<Self, fastrlp::DecodeError> {
        buf.first().ok_or(fastrlp::DecodeError::Custom("Cannot decode a log from empty bytes"))?;

        // slice out the rlp list header
        let _header = Header::decode(buf)?;

        let mut log = Log::default();

        let first = *buf
            .first()
            .ok_or(fastrlp::DecodeError::Custom("Cannot decode an address from an empty list"))?;
        // 0x0 is encoded as an empty rlp list, 0x80
        log.address = if first == 0x80u8 {
            // consume the empty list
            buf.advance(1);
            Address::zero()
        } else {
            Address::decode(buf)?
        };

        log.topics = Vec::<H256>::decode(buf)?;
        log.data.0 = bytes::Bytes::decode(buf)?;
        Ok(log)
    }
}

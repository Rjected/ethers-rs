use super::{decode_to, eip2718::TypedTransaction, eip2930::AccessList, normalize_v, rlp_opt};
use crate::types::{
    Address, Bytes, NameOrAddress, Signature, SignatureError, Transaction, U256, U64,
};
use fastrlp::length_of_length;
use rlp::{Decodable, DecoderError, RlpStream};
use thiserror::Error;

/// EIP-1559 transactions have 9 fields
const NUM_TX_FIELDS: usize = 9;

use serde::{Deserialize, Serialize};

/// An error involving an EIP1559 transaction request.
#[derive(Debug, Error)]
pub enum Eip1559RequestError {
    /// When decoding a transaction request from RLP
    #[error(transparent)]
    DecodingError(#[from] rlp::DecoderError),
    /// When recovering the address from a signature
    #[error(transparent)]
    RecoveryError(#[from] SignatureError),
}

/// Parameters for sending a transaction
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Eip1559TransactionRequest {
    /// Sender address or ENS name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,

    /// Recipient address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<NameOrAddress>,

    /// Supplied gas (None for sensible default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<U256>,

    /// Transferred value (None for no transfer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,

    /// The compiled code of a contract OR the first 4 bytes of the hash of the
    /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Bytes>,

    /// Transaction nonce (None for next available nonce)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<U256>,

    #[serde(rename = "accessList", default)]
    pub access_list: AccessList,

    #[serde(rename = "maxPriorityFeePerGas", default, skip_serializing_if = "Option::is_none")]
    /// Represents the maximum tx fee that will go to the miner as part of the user's
    /// fee payment. It serves 3 purposes:
    /// 1. Compensates miners for the uncle/ommer risk + fixed costs of including transaction in a
    /// block; 2. Allows users with high opportunity costs to pay a premium to miners;
    /// 3. In times where demand exceeds the available block space (i.e. 100% full, 30mm gas),
    /// this component allows first price auctions (i.e. the pre-1559 fee model) to happen on the
    /// priority fee.
    ///
    /// More context [here](https://hackmd.io/@q8X_WM2nTfu6nuvAzqXiTQ/1559-wallets)
    pub max_priority_fee_per_gas: Option<U256>,

    #[serde(rename = "maxFeePerGas", default, skip_serializing_if = "Option::is_none")]
    /// Represents the maximum amount that a user is willing to pay for their tx (inclusive of
    /// baseFeePerGas and maxPriorityFeePerGas). The difference between maxFeePerGas and
    /// baseFeePerGas + maxPriorityFeePerGas is “refunded” to the user.
    pub max_fee_per_gas: Option<U256>,

    #[serde(skip_serializing)]
    #[serde(default, rename = "chainId")]
    /// Chain ID (None for mainnet)
    pub chain_id: Option<U64>,
}

impl Eip1559TransactionRequest {
    /// Creates an empty transaction request with all fields left empty
    pub fn new() -> Self {
        Self::default()
    }

    // Builder pattern helpers

    /// Sets the `from` field in the transaction to the provided value
    #[must_use]
    pub fn from<T: Into<Address>>(mut self, from: T) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Sets the `to` field in the transaction to the provided value
    #[must_use]
    pub fn to<T: Into<NameOrAddress>>(mut self, to: T) -> Self {
        self.to = Some(to.into());
        self
    }

    /// Sets the `gas` field in the transaction to the provided value
    #[must_use]
    pub fn gas<T: Into<U256>>(mut self, gas: T) -> Self {
        self.gas = Some(gas.into());
        self
    }

    /// Sets the `max_priority_fee_per_gas` field in the transaction to the provided value
    #[must_use]
    pub fn max_priority_fee_per_gas<T: Into<U256>>(mut self, max_priority_fee_per_gas: T) -> Self {
        self.max_priority_fee_per_gas = Some(max_priority_fee_per_gas.into());
        self
    }

    /// Sets the `max_fee_per_gas` field in the transaction to the provided value
    #[must_use]
    pub fn max_fee_per_gas<T: Into<U256>>(mut self, max_fee_per_gas: T) -> Self {
        self.max_fee_per_gas = Some(max_fee_per_gas.into());
        self
    }

    /// Sets the `value` field in the transaction to the provided value
    #[must_use]
    pub fn value<T: Into<U256>>(mut self, value: T) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets the `data` field in the transaction to the provided value
    #[must_use]
    pub fn data<T: Into<Bytes>>(mut self, data: T) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Sets the `access_list` field in the transaction to the provided value
    #[must_use]
    pub fn access_list<T: Into<AccessList>>(mut self, access_list: T) -> Self {
        self.access_list = access_list.into();
        self
    }

    /// Sets the `nonce` field in the transaction to the provided value
    #[must_use]
    pub fn nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
        self.nonce = Some(nonce.into());
        self
    }

    /// Sets the `chain_id` field in the transaction to the provided value
    #[must_use]
    pub fn chain_id<T: Into<U64>>(mut self, chain_id: T) -> Self {
        self.chain_id = Some(chain_id.into());
        self
    }

    /// Gets the unsigned transaction's RLP encoding
    pub fn rlp(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_TX_FIELDS);
        self.rlp_base(&mut rlp);
        rlp.out().freeze().into()
    }

    /// Produces the RLP encoding of the transaction with the provided signature
    pub fn rlp_signed(&self, signature: &Signature) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_unbounded_list();
        self.rlp_base(&mut rlp);

        // if the chain_id is none we assume mainnet and choose one
        let chain_id = self.chain_id.unwrap_or_else(U64::one);

        // append the signature
        let v = normalize_v(signature.v, chain_id);
        rlp.append(&v);
        rlp.append(&signature.r);
        rlp.append(&signature.s);
        rlp.finalize_unbounded_list();
        rlp.out().freeze().into()
    }

    pub(crate) fn rlp_base(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, &self.chain_id);
        rlp_opt(rlp, &self.nonce);
        rlp_opt(rlp, &self.max_priority_fee_per_gas);
        rlp_opt(rlp, &self.max_fee_per_gas);
        rlp_opt(rlp, &self.gas);
        rlp_opt(rlp, &self.to.as_ref());
        rlp_opt(rlp, &self.value);
        rlp_opt(rlp, &self.data.as_ref().map(|d| d.as_ref()));
        rlp.append(&self.access_list);
    }

    /// Decodes fields of the request starting at the RLP offset passed. Increments the offset for
    /// each element parsed.
    #[inline]
    pub fn decode_base_rlp(rlp: &rlp::Rlp, offset: &mut usize) -> Result<Self, DecoderError> {
        let mut tx = Self::new();
        tx.chain_id = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.nonce = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.max_priority_fee_per_gas = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.max_fee_per_gas = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.gas = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.to = decode_to(rlp, offset)?;
        tx.value = Some(rlp.val_at(*offset)?);
        *offset += 1;
        let data = rlp::Rlp::new(rlp.at(*offset)?.as_raw()).data()?;
        tx.data = match data.len() {
            0 => None,
            _ => Some(Bytes::from(data.to_vec())),
        };
        *offset += 1;
        tx.access_list = rlp.val_at(*offset)?;
        *offset += 1;
        Ok(tx)
    }

    /// Decodes the given RLP into a transaction, attempting to decode its signature as well.
    pub fn decode_signed_rlp(rlp: &rlp::Rlp) -> Result<(Self, Signature), Eip1559RequestError> {
        let mut offset = 0;
        let mut txn = Self::decode_base_rlp(rlp, &mut offset)?;

        let v = rlp.val_at(offset)?;
        offset += 1;
        let r = rlp.val_at(offset)?;
        offset += 1;
        let s = rlp.val_at(offset)?;

        let sig = Signature { r, s, v };
        txn.from = Some(sig.recover(TypedTransaction::Eip1559(txn.clone()).sighash())?);

        Ok((txn, sig))
    }
}

impl Decodable for Eip1559TransactionRequest {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        Self::decode_base_rlp(rlp, &mut 0)
    }
}

impl fastrlp::Decodable for Eip1559TransactionRequest {
    fn decode(buf: &mut &[u8]) -> Result<Self, fastrlp::DecodeError> {
        // we need to decode in the right order, so let's define a struct and just derive the
        // decoding
        // [chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas_limit, destination, amount, data, access_list]
        let list_header = *buf.first().ok_or(fastrlp::DecodeError::Custom(
            "Cannot decode a transaction from an empty list",
        ))?;

        println!("tx body before header strip: {:X?}", buf);
        *buf = if list_header <= 0xf7 {
            &buf[1..]
        } else {
            let len_of_len = list_header as usize - 0xf7;
            &buf[1 + len_of_len..]
        };
        println!("tx body after header strip: {:X?}", buf);

        let mut request = Eip1559TransactionRequest::default();
        request.chain_id = Some(<bytes::Bytes as fastrlp::Decodable>::decode(buf)?[..].into());
        println!("tx body after chainid: {:X?}", buf);

        request.nonce =
            Some(<bytes::Bytes as fastrlp::Decodable>::decode(buf)?[..].into());
        println!("tx body after nonce: {:X?}", buf);
        request.max_priority_fee_per_gas =
            Some(<bytes::Bytes as fastrlp::Decodable>::decode(buf)?[..].into());
        println!("tx body after max prio: {:X?}", buf);
        request.max_fee_per_gas =
            Some(<bytes::Bytes as fastrlp::Decodable>::decode(buf)?[..].into());
        println!("tx body after max fee: {:X?}", buf);
        request.gas = Some(<bytes::Bytes as fastrlp::Decodable>::decode(buf)?[..].into());
        println!("tx body after gas: {:X?}", buf);

        let first = *buf.first().ok_or(fastrlp::DecodeError::Custom("cannot decode an address from an empty list"))?;
        // 0x0 is encoded as an empty rlp list, 0x80
        request.to = if first == 0x80u8 {
            // consume the empty list
            *buf = &buf[1..];
            None
        } else {
            Some(<NameOrAddress as fastrlp::Decodable>::decode(buf)?)
        };
        println!("tx body after to: {:X?}", buf);
        request.value =
            Some(<bytes::Bytes as fastrlp::Decodable>::decode(buf)?[..].into());
        println!("tx body after value: {:X?}", buf);

        let decoded_data = <bytes::Bytes as fastrlp::Decodable>::decode(buf)?;
        request.data = match decoded_data.len() {
            0 => None,
            _ => Some(Bytes(decoded_data)),
        };
        println!("tx body after data: {:X?}", buf);

        request.access_list = <AccessList as fastrlp::Decodable>::decode(buf)?;
        println!("tx body after access list: {:X?}", buf);
        Ok(request)
    }
}


impl fastrlp::Encodable for Eip1559TransactionRequest {
    fn length(&self) -> usize {
        // add each of the fields' rlp encoded lengths
        let mut length: usize = 0;
        // the max value for a single byte to represent itself is 0x7f
        let max_for_header = U256::from(0x7fu8);
        // the number of rlp string headers - each U256 can be either a single byte (and is < 0x7f)
        // or less than 32
        let mut headers_len = 0;

        length += self.chain_id.unwrap_or_else(U64::one).as_u64().length();

        length += 32 - self.nonce.unwrap_or_default().leading_zeros() as usize / 8;
        headers_len += if self.nonce.unwrap_or_default() < max_for_header { 0 } else { 1 };

        length += 32 - self.max_priority_fee_per_gas.unwrap_or_default().leading_zeros() as usize / 8;
        headers_len += if self.max_priority_fee_per_gas.unwrap_or_default() < max_for_header { 0 } else { 1 };

        length += 32 - self.max_fee_per_gas.unwrap_or_default().leading_zeros() as usize / 8;
        headers_len += if self.max_fee_per_gas.unwrap_or_default() < max_for_header { 0 } else { 1 };

        length += 32 - self.gas.unwrap_or_default().leading_zeros() as usize / 8;
        headers_len += if self.gas.unwrap_or_default() < max_for_header { 0 } else { 1 };

        let to_addr =
            self.to.to_owned().unwrap_or_else(|| NameOrAddress::Address(Address::default()));
        length += to_addr.length();

        length += 32 - self.value.unwrap_or_default().leading_zeros() as usize / 8;
        headers_len += if self.value.unwrap_or_default() < max_for_header { 0 } else { 1 };

        length += self.data.to_owned().unwrap_or_default().0.length();

        length += self.access_list.length();

        length += headers_len;

        length
    }

    fn encode(&self, out: &mut dyn bytes::BufMut) {
        // [chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas_limit, destination, amount, data, access_list]
        let mut uint_container = [0x00; 32];

        let encoding_len = self.length();
        // have to implement header encoding rules for lists since the transaction will be encoded
        // as a list
        if encoding_len <= 55 {
            let header = self.length() as u8 + 0xc0;
            out.put_u8(header);
        } else {
            let len_of_len = length_of_length(encoding_len);
            out.put_uint(encoding_len as u64, len_of_len);
            out.put_u8(0xf7 + len_of_len as u8);
        }

        // if the chain_id is none we assume mainnet and choose one
        self.chain_id.unwrap_or_else(U64::one).as_u64().encode(out);

        let nonce = self.nonce.unwrap_or_default();
        nonce.to_big_endian(&mut uint_container[..]);
        let nonce_bytes = &uint_container[31 - nonce.bits() as usize / 8..];
        nonce_bytes.encode(out);

        let max_priority_fee_per_gas = self.max_priority_fee_per_gas.unwrap_or_default();
        max_priority_fee_per_gas.to_big_endian(&mut uint_container[..]);
        let max_priority_fee_per_gas_bytes = &uint_container[31 - max_priority_fee_per_gas.bits() as usize / 8..];
        max_priority_fee_per_gas_bytes.encode(out);

        let max_fee_per_gas = self.max_fee_per_gas.unwrap_or_default();
        max_fee_per_gas.to_big_endian(&mut uint_container[..]);
        let max_fee_per_gas_bytes = &uint_container[31 - max_fee_per_gas.bits() as usize / 8..];
        max_fee_per_gas_bytes.encode(out);

        let gas = self.gas.unwrap_or_default();
        gas.to_big_endian(&mut uint_container[..]);
        let gas_bytes = &uint_container[31 - gas.bits() as usize / 8..];
        gas_bytes.encode(out);

        let to_addr =
            self.to.to_owned().unwrap_or_else(|| NameOrAddress::Address(Address::default()));
        to_addr.encode(out);

        let value = self.value.unwrap_or_default();
        value.to_big_endian(&mut uint_container[..]);
        let value_bytes = &uint_container[31 - value.bits() as usize / 8..];
        value_bytes.encode(out);

        self.data.to_owned().unwrap_or_default().0.encode(out);

        self.access_list.encode(out);
    }
}

impl From<Eip1559TransactionRequest> for super::request::TransactionRequest {
    fn from(tx: Eip1559TransactionRequest) -> Self {
        Self {
            from: tx.from,
            to: tx.to,
            gas: tx.gas,
            gas_price: tx.max_fee_per_gas,
            value: tx.value,
            data: tx.data,
            nonce: tx.nonce,
            #[cfg(feature = "celo")]
            fee_currency: None,
            #[cfg(feature = "celo")]
            gateway_fee_recipient: None,
            #[cfg(feature = "celo")]
            gateway_fee: None,
            chain_id: tx.chain_id,
        }
    }
}

impl From<&Transaction> for Eip1559TransactionRequest {
    fn from(tx: &Transaction) -> Eip1559TransactionRequest {
        Eip1559TransactionRequest {
            from: Some(tx.from),
            to: tx.to.map(NameOrAddress::Address),
            gas: Some(tx.gas),
            value: Some(tx.value),
            data: Some(Bytes(tx.input.0.clone())),
            nonce: Some(tx.nonce),
            access_list: tx.access_list.clone().unwrap_or_default(),
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
            max_fee_per_gas: tx.max_fee_per_gas,
            chain_id: tx.chain_id.map(|x| U64::from(x.as_u64())),
        }
    }
}

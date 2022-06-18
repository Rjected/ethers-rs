use std::cmp::Ordering;

use crate::types::Address;
use rlp::{Decodable, Encodable, RlpStream};
use serde::{ser::Error as SerializationError, Deserialize, Deserializer, Serialize, Serializer};

/// ENS name or Ethereum Address. Not RLP encoded/serialized if it's a name
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NameOrAddress {
    /// An ENS Name (format does not get checked)
    Name(String),
    /// An Ethereum Address
    Address(Address),
}

// Only RLP encode the Address variant since it doesn't make sense to ever RLP encode
// an ENS name
impl fastrlp::Encodable for NameOrAddress {
    fn length(&self) -> usize {
        match self {
            // encoding doesn't make sense for ENS names, so let's return 0 as the length
            Self::Name(_) => 0,
            Self::Address(addr) => {
                if *addr == Address::zero() {
                    1
                } else {
                    <Address as fastrlp::Encodable>::length(addr)
                }
            }
        }
    }
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        if let NameOrAddress::Address(addr) = self {
            if *addr == Address::zero() {
                out.put_u8(0x80);
            } else {
                <Address as fastrlp::Encodable>::encode(addr, out);
            }
        }
    }
}

impl fastrlp::Decodable for NameOrAddress {
    fn decode(buf: &mut &[u8]) -> Result<Self, fastrlp::DecodeError> {
        let addr = <Address as fastrlp::Decodable>::decode(buf)?;
        Ok(Self::Address(addr))
    }
}

// Only RLP encode the Address variant since it doesn't make sense to ever RLP encode
// an ENS name
impl Encodable for &NameOrAddress {
    fn rlp_append(&self, s: &mut RlpStream) {
        if let NameOrAddress::Address(inner) = self {
            inner.rlp_append(s);
        }
    }
}

impl Encodable for NameOrAddress {
    fn rlp_append(&self, s: &mut RlpStream) {
        if let NameOrAddress::Address(inner) = self {
            inner.rlp_append(s);
        }
    }
}

impl Decodable for NameOrAddress {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        // An address (H160) is 20 bytes, so let's only accept 20 byte rlp string encodings.
        if !rlp.is_data() {
            return Err(rlp::DecoderError::RlpExpectedToBeData)
        }

        // the data needs to be 20 bytes long
        match 20.cmp(&rlp.size()) {
            Ordering::Less => Err(rlp::DecoderError::RlpIsTooShort),
            Ordering::Greater => Err(rlp::DecoderError::RlpIsTooBig),
            Ordering::Equal => {
                let rlp_data = rlp.data()?;
                Ok(NameOrAddress::Address(Address::from_slice(rlp_data)))
            }
        }
    }
}

impl From<&str> for NameOrAddress {
    fn from(s: &str) -> Self {
        NameOrAddress::Name(s.to_owned())
    }
}

impl From<Address> for NameOrAddress {
    fn from(s: Address) -> Self {
        NameOrAddress::Address(s)
    }
}

// Only serialize the Address variant since it doesn't make sense to ever serialize
// an ENS name
impl Serialize for NameOrAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            NameOrAddress::Address(addr) => addr.serialize(serializer),
            NameOrAddress::Name(name) => Err(SerializationError::custom(format!(
                "cannot serialize ENS name {}, must be address",
                name
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for NameOrAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = Address::deserialize(deserializer)?;

        Ok(NameOrAddress::Address(inner))
    }
}

#[cfg(test)]
mod tests {
    use rlp::Rlp;

    use super::*;

    #[test]
    fn rlp_name_not_serialized() {
        let name = NameOrAddress::Name("ens.eth".to_string());

        let mut rlp = RlpStream::new();
        name.rlp_append(&mut rlp);
        assert!(rlp.is_empty());

        let mut rlp = RlpStream::new();
        name.rlp_append(&mut rlp);
        assert!(rlp.is_empty());
    }

    #[test]
    fn rlp_address_serialized() {
        let addr = "f02c1c8e6114b1dbe8937a39260b5b0a374432bb".parse().unwrap();
        let union = NameOrAddress::Address(addr);

        let mut expected = RlpStream::new();
        addr.rlp_append(&mut expected);

        let mut rlp = RlpStream::new();
        union.rlp_append(&mut rlp);
        assert_eq!(rlp.as_raw(), expected.as_raw());

        let mut rlp = RlpStream::new();
        union.rlp_append(&mut rlp);
        assert_eq!(rlp.as_raw(), expected.as_raw());
    }

    #[test]
    fn rlp_address_deserialized() {
        let addr = "3dd6f334b732d23b51dfbee2070b40bbd1a97a8f".parse().unwrap();
        let expected = NameOrAddress::Address(addr);

        let mut rlp = RlpStream::new();
        rlp.append(&addr);
        let rlp_bytes = &rlp.out().freeze()[..];
        let data = Rlp::new(rlp_bytes);
        let name = NameOrAddress::decode(&data).unwrap();

        assert_eq!(name, expected);
    }

    #[test]
    fn fastrlp_address_deserialized() {
        let addr = "3dd6f334b732d23b51dfbee2070b40bbd1a97a8f".parse().unwrap();
        let expected = NameOrAddress::Address(addr);
        let mut rlp_bytes = vec![];
        <NameOrAddress as fastrlp::Encodable>::encode(&expected, &mut rlp_bytes);

        let decoded_addr =
            <NameOrAddress as fastrlp::Decodable>::decode(&mut &rlp_bytes[..]).unwrap();
        assert_eq!(decoded_addr, expected);
    }

    #[test]
    fn fastrlp_name_not_serialized() {
        let name = NameOrAddress::Name("ens.eth".to_string());
        let mut rlp_bytes = vec![];
        <NameOrAddress as fastrlp::Encodable>::encode(&name, &mut rlp_bytes);
        let expected: Vec<u8> = vec![];
        assert_eq!(rlp_bytes, expected);
    }

    #[test]
    fn serde_name_not_serialized() {
        let name = NameOrAddress::Name("ens.eth".to_string());
        bincode::serialize(&name).unwrap_err();
    }

    #[test]
    fn serde_address_serialized() {
        let addr = "f02c1c8e6114b1dbe8937a39260b5b0a374432bb".parse().unwrap();
        let union = NameOrAddress::Address(addr);

        assert_eq!(bincode::serialize(&addr).unwrap(), bincode::serialize(&union).unwrap(),);
    }
}

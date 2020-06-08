use blake3::Hash;
use serde::{Serialize, Serializer, Deserialize/*, Deserializer*/};
//use serde::de::{Visitor, Error};
//use std::fmt;
//#[macro_use(serde)]

// Follows: https://serde.rs/remote-derive.html

#[derive(/*Serialize,*/ Deserialize, Debug)]
//#[serde(remote = "Hash")]
pub struct SerializableHash([u8; blake3::OUT_LEN]);

impl Serialize for SerializableHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

// Provide a conversion to construct the remote type.
impl From<SerializableHash> for Hash {
    fn from(hash: SerializableHash) -> Hash {
        hash.0.into()
    }
}

impl From<Hash> for SerializableHash {
    fn from(hash: Hash) -> SerializableHash {
        SerializableHash(hash.into())
    }
}

impl PartialEq<Hash> for SerializableHash {
    fn eq(&self, other: &Hash) -> bool {
        &self.0 == other.as_bytes() // I don't care about constant time here!
    }
}

// impl<'de> Visitor<'de> for SerializableHash {
//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         write!(formatter, "a byte array containing at least {} bytes", blake3::OUT_LEN)
//     }

//     fn visit_bytes<E>(self, v: &[u8]) -> Result<[u8; blake3::OUT_LEN], E>
//     where
//         E: Error,
//     {
//         let mut result: [u8; blake3::OUT_LEN];
//         result.clone_from_slice(v);
//         Ok(result)
//     }
// }

// impl<'de> Deserialize<'de> for SerializableHash {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_bytes(SerializableHash)?;
//     }
// }

// impl Serialize for Option<Hash> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         match *self {
//             None => serializer.serialize_none(),
//             Some(x) => serializer.serialize_some(x.as_bytes())
//         }
//     }
// }
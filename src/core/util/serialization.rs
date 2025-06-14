use serde::{de::DeserializeOwned, Serialize};

/// Serialize any serde serializable object to bytes using bincode.
pub fn to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(value)
}

/// Deserialize bytes back into an object using bincode.
pub fn from_bytes<T: DeserializeOwned>(data: &[u8]) -> Result<T, bincode::Error> {
    bincode::deserialize(data)
}

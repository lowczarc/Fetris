use serde::{Deserialize, Serialize};
use std::io::Read;

pub type DeserializeError = bincode::Error;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientRequest {
    Message(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerRequest {
    Message(String, String),
}

impl ClientRequest {
    pub fn into_bytes(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn from_bytes(encoded: &[u8]) -> Self {
        bincode::deserialize(encoded).unwrap()
    }

    pub fn from_reader<R: Read>(reader: R) -> Result<Self, DeserializeError> {
        bincode::deserialize_from(reader)
    }
}

impl ServerRequest {
    pub fn into_bytes(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn from_bytes(encoded: &[u8]) -> Self {
        bincode::deserialize(encoded).unwrap()
    }

    pub fn from_reader<R: Read>(reader: R) -> Result<Self, DeserializeError> {
        bincode::deserialize_from(reader)
    }
}

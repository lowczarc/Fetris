use serde::{Deserialize, Serialize};
use std::io::Read;

pub mod game;
pub mod rotation_tetrimino;
pub mod tetrimino;
pub mod tetrimino_bag;

pub type DeserializeError = bincode::Error;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientRequest {
    SetName(String),
    AskForAGame,
    Input(game::Input),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ServerRequest {
    BadRequest,
    GameReady(game::PlayerGame),
    Action(
        game::GameAction,
        game::PlayerGame,
        Vec<game::PlayerMinimalInfos>,
    ),
    GameOver,
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

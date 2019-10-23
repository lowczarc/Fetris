use std::net::SocketAddr;

use crate::game::pools::PoolState;

#[derive(Clone)]
pub struct Player {
    name: String,
    pool: PoolState,
    socket: SocketAddr,
}

impl Player {
    pub fn new(socket: SocketAddr) -> Self {
        Self {
            name: "Anonyme".into(),
            pool: PoolState::None,
            socket,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn pool(&self) -> PoolState {
        self.pool.clone()
    }

    pub fn change_pool(&mut self, pool: PoolState) {
        self.pool = pool;
    }

    pub fn socket(&self) -> &SocketAddr {
        &self.socket
    }
}

pub enum TetriminoType {
    I,
    L,
    J,
    O,
    Z,
    S,
}

pub struct Tetrimino {
    ttype: TetriminoType,
    rotation: (bool, bool),
}

pub struct PoolPlayer {
    name: String,
    matrix: [[bool; 40]; 10],
    current_tetrimino: Tetrimino,
    stocked_tetrimino: Tetrimino,
    pending_tetriminos: Vec<Tetrimino>,
}

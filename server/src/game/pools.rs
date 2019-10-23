use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use fetris_protocol::ServerRequest;

use crate::game::players::Player;
use crate::network::StreamList;

pub type PoolId = Instant;

fn generate_pool_id() -> PoolId {
    Instant::now()
}

pub struct Pool<'a> {
    players: HashMap<SocketAddr, ()>,
    stream_list: &'a StreamList,
    last_call: Instant,
    call_every: Duration,
}

impl<'a> Pool<'a> {
    pub fn create(
        players: &mut HashMap<SocketAddr, Player>,
        stream_list: &'a StreamList,
        pool_players: HashMap<SocketAddr, ()>,
    ) -> (PoolId, Self) {
        let id = generate_pool_id();
        for (socket, _) in pool_players.iter() {
            let player = players.get_mut(socket).unwrap();

            let _ = stream_list.send_to(socket, ServerRequest::GameReady);
            player.change_pool(PoolState::Pool(id));
        }

        (
            id,
            Self {
                players: pool_players,
                stream_list,
                last_call: Instant::now(),
                call_every: Duration::from_secs(2),
            },
        )
    }

    pub fn is_to_call(&self) -> bool {
        Instant::now().duration_since(self.last_call) >= self.call_every
    }

    pub fn update(&mut self) {
        println!("test");
        self.last_call = Instant::now();
    }

    pub fn remove_user(&mut self, socket: &SocketAddr) {
        self.players.remove(socket);
    }

    pub fn len(&self) -> usize {
        self.players.len()
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum PoolState {
    Pool(PoolId),
    PendingPool,
    None,
}

pub const POOL_SIZE: usize = 2;

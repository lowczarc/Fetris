use rand::{self, Rng};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use fetris_protocol::game::{Direction, GameAction, Input, PlayerGame, TetriminoType};
use fetris_protocol::ServerRequest;

use crate::game::players::Player;
use crate::network::StreamList;

pub type PoolId = Instant;

fn generate_pool_id() -> PoolId {
    Instant::now()
}

fn tetrimino_rand() -> TetriminoType {
    match rand::thread_rng().gen_range(0, 7) {
        0 => TetriminoType::I,
        1 => TetriminoType::J,
        2 => TetriminoType::L,
        3 => TetriminoType::O,
        4 => TetriminoType::S,
        5 => TetriminoType::T,
        6 => TetriminoType::Z,
        _ => TetriminoType::None,
    }
}

pub struct PlayerInfos {
    pub player: PlayerGame,
    pub last_call: Instant,
}

impl PlayerInfos {
    pub fn new(player: PlayerGame) -> Self {
        Self {
            player,
            last_call: Instant::now(),
        }
    }
}

pub struct Pool<'a> {
    players: HashMap<SocketAddr, PlayerInfos>,
    stream_list: &'a StreamList,
    call_every: Duration,
}

impl<'a> Pool<'a> {
    pub fn create(
        players: &mut HashMap<SocketAddr, Player>,
        stream_list: &'a StreamList,
        pool_sockets: HashMap<SocketAddr, ()>,
    ) -> (PoolId, Self) {
        let id = generate_pool_id();
        let mut pool_players = HashMap::new();
        for (socket, _) in pool_sockets.iter() {
            let player = players.get_mut(socket).unwrap();
            let player_game = PlayerGame::new(player.name().into());

            let _ = stream_list.send_to(socket, ServerRequest::GameReady(player_game.clone()));
            pool_players.insert(socket.clone(), PlayerInfos::new(player_game));
            player.change_pool(PoolState::Pool(id));
        }

        (
            id,
            Self {
                players: pool_players,
                stream_list,
                call_every: Duration::from_millis(500),
            },
        )
    }

    pub fn remove_user(&mut self, socket: &SocketAddr) {
        self.players.remove(socket);
    }

    pub fn update(&mut self) {
        for (socket, player) in self.players.iter_mut() {
            if Instant::now().duration_since(player.last_call) < self.call_every {
                continue;
            }
            let matrix = player.player.matrix().clone();
            if let Some(tetrimino) = player.player.current_tetrimino_mut() {
                if tetrimino.can_move_to(&matrix, Direction::Down) {
                    tetrimino.apply_direction(Direction::Down);
                    let _ = self.stream_list.send_to(
                        socket,
                        ServerRequest::Action(
                            GameAction::MoveCurrentTetrimino(Direction::Down),
                            player.player.clone(),
                        ),
                    );
                } else {
                    player.player.place_current_tetrimino();
                }
            } else {
                player.player.change_current_tetrimino(tetrimino_rand());
                let _ = self.stream_list.send_to(
                    socket,
                    ServerRequest::Action(GameAction::NewTetrimino, player.player.clone()),
                );
            }
            player.last_call = Instant::now();
        }
    }

    pub fn handle_player_input(&mut self, socket: &SocketAddr, input: Input) {
        let player = self.players.get_mut(socket).unwrap();
        match input {
            Input::Left => {
                let matrix = player.player.matrix().clone();
                if let Some(tetrimino) = player.player.current_tetrimino_mut() {
                    if tetrimino.can_move_to(&matrix, Direction::Left) {
                        tetrimino.apply_direction(Direction::Left);
                        let _ = self.stream_list.send_to(
                            &socket,
                            ServerRequest::Action(
                                GameAction::MoveCurrentTetrimino(Direction::Left),
                                player.player.clone(),
                            ),
                        );
                    }
                    player.last_call = Instant::now();
                }
            }
            Input::Right => {
                let matrix = player.player.matrix().clone();
                if let Some(tetrimino) = player.player.current_tetrimino_mut() {
                    if tetrimino.can_move_to(&matrix, Direction::Right) {
                        tetrimino.apply_direction(Direction::Right);
                        let _ = self.stream_list.send_to(
                            &socket,
                            ServerRequest::Action(
                                GameAction::MoveCurrentTetrimino(Direction::Right),
                                player.player.clone(),
                            ),
                        );
                    }
                    player.last_call = Instant::now();
                }
            }
            Input::FastMove => {
                let matrix = player.player.matrix().clone();
                if let Some(tetrimino) = player.player.current_tetrimino_mut() {
                    loop {
                        if tetrimino.can_move_to(&matrix, Direction::Down) {
                            tetrimino.apply_direction(Direction::Down);
                        } else {
                            player.player.place_current_tetrimino();
                            break;
                        }
                    }
                    let _ = self.stream_list.send_to(
                        socket,
                        ServerRequest::Action(
                            GameAction::MoveCurrentTetrimino(Direction::Down),
                            player.player.clone(),
                        ),
                    );
                    player.last_call = Instant::now();
                }
            }
            Input::Rotate => {
                let matrix = player.player.matrix().clone();
                if let Some(tetrimino) = player.player.current_tetrimino_mut() {
                    if tetrimino.rotate(&matrix) {
                        let _ = self.stream_list.send_to(
                            socket,
                            ServerRequest::Action(GameAction::Rotate, player.player.clone()),
                        );
                        player.last_call = Instant::now();
                    }
                }
            }
            _ => {}
        }
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

pub const POOL_SIZE: usize = 1;

use rand::{self, Rng};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use fetris_protocol::game::{Direction, GameAction, Input, PlayerGame, PlayerMinimalInfos};
use fetris_protocol::ServerRequest;

use crate::game::players::Player;
use crate::network::StreamList;

pub type PoolId = Instant;

fn generate_pool_id() -> PoolId {
    Instant::now()
}

pub struct PlayerInfos {
    pub player: PlayerGame,
    pub last_call: Instant,
    pub garbage_received: u32,
    pub accelerate: bool,
    pub dead: bool,
}

impl PlayerInfos {
    pub fn new(player: PlayerGame) -> Self {
        Self {
            player,
            last_call: Instant::now(),
            garbage_received: 0,
            accelerate: false,
            dead: false,
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

    pub fn user_list(&self) -> Vec<PlayerMinimalInfos> {
        self.players
            .values()
            .map(|elem| PlayerMinimalInfos {
                name: elem.player.name().to_string(),
                dead: elem.dead,
            })
            .collect()
    }

    pub fn send_garbage(&mut self, sender: &SocketAddr, row_broken: u32, is_t_spin: bool) {
        let player_list = self.user_list();
        let garbage_to_send = match (is_t_spin, row_broken) {
            (false, 2) => 1,
            (false, 3) => 2,
            (false, 4) => 4,
            (true, x) => 2 * x,
            (_, _) => 0,
        };

        if garbage_to_send == 0 {
            return;
        }

        let mut receiver: Option<SocketAddr> = None;
        let mut last_receiver_garbage = 0;

        for (addr, player) in self.players.iter() {
            if addr == sender || player.dead {
                continue;
            }
            if receiver.is_none() || last_receiver_garbage > player.garbage_received {
                receiver = Some(addr.clone());
                last_receiver_garbage = player.garbage_received;
            }
        }

        println!(
            "Garbage to send: {}, Receiver: {:?}, t-spin: {}",
            garbage_to_send, receiver, is_t_spin
        );

        if let Some(addr) = receiver {
            let player = self.players.get_mut(&addr).unwrap();
            let hole_position = rand::thread_rng().gen_range(0, 10);

            player.garbage_received += garbage_to_send;
            for _ in 0..garbage_to_send {
                player.player.add_garbage(hole_position);
            }
            let _ = self.stream_list.send_to(
                &addr,
                ServerRequest::Action(
                    GameAction::GetGarbage(garbage_to_send),
                    player.player.clone(),
                    player_list.clone(),
                ),
            );
        }
    }

    pub fn update(&mut self) {
        let mut garbage: Vec<(SocketAddr, u32, bool)> = Vec::new();
        let player_list = self.user_list();
        for (socket, player) in self.players.iter_mut() {
            if player.dead
                || (Instant::now().duration_since(player.last_call) < self.call_every
                    && !player.accelerate)
            {
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
                            player_list.clone(),
                        ),
                    );
                } else if !player.accelerate {
                    let is_t_spin = !tetrimino.can_move_to(&matrix, Direction::Left)
                        && !tetrimino.can_move_to(&matrix, Direction::Right)
                        && !tetrimino.can_move_to(&matrix, Direction::Up);
                    let row_broken = player.player.place_current_tetrimino();

                    garbage.push((socket.clone(), row_broken.len() as u32, is_t_spin));
                }
            } else {
                player.player.new_tetrimino();
                if !player.player.current_tetrimino().unwrap().is_valid(&matrix) {
                    player.dead = true;
                    let _ = self.stream_list.send_to(socket, ServerRequest::GameOver);
                    println!("{} is dead", socket);
                } else {
                    let _ = self.stream_list.send_to(
                        socket,
                        ServerRequest::Action(
                            GameAction::NewTetrimino,
                            player.player.clone(),
                            player_list.clone(),
                        ),
                    );
                }
            }

            if player.accelerate {
                player.accelerate = false;
            }
            player.last_call = Instant::now();
        }
        for (addr, row_broken, is_t_spin) in garbage.into_iter() {
            self.send_garbage(&addr, row_broken, is_t_spin);
        }
    }

    pub fn handle_player_input(&mut self, socket: &SocketAddr, input: Input) {
        let mut garbage: Option<(SocketAddr, u32, bool)> = None;
        let player_list = self.user_list();
        let player = self.players.get_mut(socket).unwrap();
        if player.dead {
            return;
        }
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
                                player_list.clone(),
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
                                player_list.clone(),
                            ),
                        );
                    }
                    player.last_call = Instant::now();
                }
            }
            Input::FastMove => {
                let matrix = player.player.matrix().clone();
                if let Some(tetrimino) = player.player.current_tetrimino_mut() {
                    while tetrimino.can_move_to(&matrix, Direction::Down) {
                        tetrimino.apply_direction(Direction::Down);
                    }

                    let is_t_spin = !tetrimino.can_move_to(&matrix, Direction::Left)
                        && !tetrimino.can_move_to(&matrix, Direction::Right)
                        && !tetrimino.can_move_to(&matrix, Direction::Up);
                    let row_broken = player.player.place_current_tetrimino();

                    garbage = Some((socket.clone(), row_broken.len() as u32, is_t_spin));

                    let _ = self.stream_list.send_to(
                        socket,
                        ServerRequest::Action(
                            GameAction::MoveCurrentTetrimino(Direction::Down),
                            player.player.clone(),
                            player_list.clone(),
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
                            ServerRequest::Action(
                                GameAction::Rotate,
                                player.player.clone(),
                                player_list.clone(),
                            ),
                        );
                        player.last_call = Instant::now();
                    }
                }
            }
            Input::StockTetrimino => {
                if player.player.current_tetrimino().is_some() {
                    player.player.stock_current_tetrimino();
                    let _ = self.stream_list.send_to(
                        socket,
                        ServerRequest::Action(
                            GameAction::StockTetrimino,
                            player.player.clone(),
                            player_list.clone(),
                        ),
                    );
                    player.last_call = Instant::now();
                }
            }
            Input::Acceleration => {
                player.accelerate = true;
            }
        }

        if let Some((addr, row_broken, is_t_spin)) = garbage {
            self.send_garbage(&addr, row_broken, is_t_spin);
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

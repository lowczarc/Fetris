use std::{
    io::{stdin, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    time,
};
use termion::{event::Key, input::TermRead};

use crate::{client_server_showdown::ActionsQueues, config::Config};
use fetris_protocol::{
    actions,
    game::{Direction, GameAction, Input, PlayerGame},
    ClientRequest,
};

fn input_to_action(input: Input) -> GameAction {
    match input {
        Input::Left => GameAction::MoveCurrentTetrimino(Direction::Left),
        Input::Right => GameAction::MoveCurrentTetrimino(Direction::Right),
        Input::FastMove => GameAction::MoveCurrentTetrimino(Direction::FastDown),
        Input::Rotate => GameAction::Rotate,
        Input::StockTetrimino => GameAction::StockTetrimino,
        Input::Acceleration => GameAction::MoveCurrentTetrimino(Direction::Down),
        Input::Fall => panic!("Unexpected Fall Input"),
    }
}

pub fn keyboard_listen(
    mut stream: TcpStream,
    action_queues: Arc<Mutex<ActionsQueues>>,
    config: Config,
    game_board: Arc<Mutex<Option<PlayerGame>>>,
    last_action: Arc<Mutex<time::Instant>>,
) {
    let config = config.to_hashmap();
    let stdin = stdin();
    for c in stdin.keys() {
        let c = c.unwrap();
        if c == Key::Ctrl('c') {
            println!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            break;
        }
        if let Some(input) = config.get(&c) {
            if stream
                .write(&ClientRequest::Input(*input).into_bytes())
                .is_err()
            {
                //break;
            }
            {
                let mut action_queues = action_queues.lock().unwrap();
                let board = game_board.lock().unwrap();
                if let Some(board) = &*board {
                    let action_result =
                        action_queues.action_result(board, input_to_action(input.clone()));
                    if action_result != Err(actions::ApplyActionError::InvalidActionNoResetTimer) {
                        let mut last_action = last_action.lock().unwrap();
                        *last_action = time::Instant::now();
                    }
                    if action_result.is_ok() {
                        action_queues.push_client_action(input_to_action(*input));
                    }
                }
            }
        }
        if stream.peer_addr().is_err() {
            //break;
        }
    }
}

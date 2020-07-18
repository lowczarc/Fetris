use std::{
    io::stdout,
    net::TcpStream,
    sync::{Arc, Mutex},
    thread, time,
};
use termion;
use termion::raw::IntoRawMode;

use fetris_protocol::{actions, game::PlayerGame, ServerRequest};

use crate::client_server_showdown::ActionsQueues;

pub fn launch_server_receiver_thread(
    reader: TcpStream,
    action_queues: Arc<Mutex<ActionsQueues>>,
    game_board: Arc<Mutex<Option<PlayerGame>>>,
) {
    thread::spawn(move || loop {
        if let Ok(request) = ServerRequest::from_reader(&reader) {
            if request == ServerRequest::GameOver {
                let mut board = game_board.lock().unwrap();
                *board = None;

                println!("{}-------------", termion::cursor::Goto(4, 12));
                println!("{}| Game Over |", termion::cursor::Goto(4, 13));
                println!("{}-------------", termion::cursor::Goto(4, 14));
            }
            match request {
                ServerRequest::MinifiedAction(action) => {
                    let mut action_queues = action_queues.lock().unwrap();
                    let mut board = game_board.lock().unwrap();
                    let _ = actions::apply_action(board.as_mut().unwrap(), action.clone());
                    action_queues.push_server_action(action);
                }
                _ => {}
            }
        } else {
            break;
        }
    });
}

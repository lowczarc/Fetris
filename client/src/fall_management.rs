use std::{
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
    thread, time,
};

use crate::client_server_showdown::ActionsQueues;
use fetris_protocol::{
    game::{Direction, GameAction, Input, PlayerGame},
    ClientRequest,
};

pub fn fall_management_thread(
    mut stream: TcpStream,
    action_queues: Arc<Mutex<ActionsQueues>>,
    last_action: Arc<Mutex<time::Instant>>,
    board: Arc<Mutex<Option<PlayerGame>>>,
    falling_interval: u16,
) {
    let falling_interval = time::Duration::from_millis(falling_interval.into());

    thread::spawn(move || loop {
        {
            let mut last_action = last_action.lock().unwrap();
            if (time::Instant::now().duration_since(*last_action) >= falling_interval) {
                if stream
                    .write(&ClientRequest::Input(Input::Fall).into_bytes())
                    .is_err()
                {
                    break;
                }
                *last_action = time::Instant::now();
                let mut action_queues = action_queues.lock().unwrap();
                action_queues.push_client_action(GameAction::Fall);
            }
        }
        thread::sleep(time::Duration::from_millis(15));
    });
}

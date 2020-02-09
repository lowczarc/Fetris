use fetris_protocol::{actions, game::PlayerGame, ClientRequest, ServerRequest};
use std::env;
use std::io::{stdout, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time;
use termion;
use termion::raw::IntoRawMode;

mod client_server_showdown;
mod config;
mod fall_management;
mod keyboard_listener;
mod print;
mod server_receiver;

use client_server_showdown::ActionsQueues;
use config::Config;

fn main() -> Result<(), std::io::Error> {
    let config = if let Some(config) = Config::from_path("config.toml") {
        config
    } else {
        Config::default()
    };

    if env::args().len() != 2 {
        println!("Usage: fetris server_address");
        return Ok(());
    }

    let mut stream = TcpStream::connect(env::args().nth(1).unwrap())?;

    let _hide_cursor = termion::cursor::HideCursor::from(stdout());

    let game_board: Arc<Mutex<Option<PlayerGame>>> = Arc::new(Mutex::new(None));
    let printable_game: Arc<Mutex<Option<PlayerGame>>> = Arc::new(Mutex::new(None));
    let action_queues: Arc<Mutex<ActionsQueues>> = Arc::new(Mutex::new(ActionsQueues::new()));
    let last_action = Arc::new(Mutex::new(time::Instant::now()));

    stream.write(&ClientRequest::AskForAGame.into_bytes())?;

    println!(
        "{}{}Waiting for other players ...",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
    );

    let falling_interval = {
        let (game, falling_interval) = if let Ok(ServerRequest::GameReady(game, update_time)) =
            ServerRequest::from_reader(&stream)
        {
            (game, update_time)
        } else {
            panic!("Invalid first server request");
        };
        let mut board = game_board.lock().unwrap();
        *board = Some(game);
        falling_interval
    };

    let _stdout = stdout().into_raw_mode().unwrap();
    print::launch_print_thread(game_board.clone(), action_queues.clone());
    server_receiver::launch_server_receiver_thread(
        stream.try_clone().unwrap(),
        action_queues.clone(),
        game_board.clone(),
    );

    fall_management::fall_management_thread(
        stream.try_clone().unwrap(),
        action_queues.clone(),
        last_action.clone(),
        game_board.clone(),
        falling_interval,
    );

    keyboard_listener::keyboard_listen(stream, action_queues, config, game_board, last_action);
    Ok(())
}

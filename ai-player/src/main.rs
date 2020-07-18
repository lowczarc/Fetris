use fetris_protocol::{game::PlayerGame, ClientRequest, ServerRequest};
use std::env;
use std::io::{stdout, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use termion;

mod agent;
mod client_server_showdown;
mod print;
mod server_receiver;

use client_server_showdown::ActionsQueues;

fn main() -> Result<(), std::io::Error> {
    if env::args().len() != 2 {
        println!("Usage: fetris server_address");
        return Ok(());
    }

    let mut stream = TcpStream::connect(env::args().nth(1).unwrap())?;

    let _hide_cursor = termion::cursor::HideCursor::from(stdout());

    let game_board: Arc<Mutex<Option<PlayerGame>>> = Arc::new(Mutex::new(None));
    let action_queues: Arc<Mutex<ActionsQueues>> = Arc::new(Mutex::new(ActionsQueues::new()));

    stream.write(&ClientRequest::AskForAGame.into_bytes())?;

    println!(
        "{}{}Waiting for other players ...",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
    );

    {
        let game =
            if let Ok(ServerRequest::GameReady(game, _)) = ServerRequest::from_reader(&stream) {
                game
            } else {
                panic!("Invalid first server request");
            };
        let mut board = game_board.lock().unwrap();
        *board = Some(game);
    }

    print::launch_print_thread(game_board.clone(), action_queues.clone());
    server_receiver::launch_server_receiver_thread(
        stream.try_clone().unwrap(),
        action_queues.clone(),
        game_board.clone(),
    );

    agent::agent_thread(stream, game_board);
    Ok(())
}

use std::net::TcpListener;
use std::sync::mpsc;
use std::{env, thread};

mod client_handler;
mod game;
mod network;

const DEFAULT_PORT: &str = "3001";

fn main() -> Result<(), std::io::Error> {
    let port = if let Ok(port) = env::var("PORT") {
        port
    } else {
        DEFAULT_PORT.into()
    };

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    let stream_list = network::StreamList::new();
    let (sender, receiver) = mpsc::channel::<network::NetworkPacket>();

    {
        let stream_list = stream_list.clone();

        thread::spawn(move || game::game_main_thread(stream_list, receiver));
    }

    for stream in listener.incoming() {
        let stream_list = stream_list.clone();
        let sender = sender.clone();

        thread::spawn(move || client_handler::client_handler(stream.unwrap(), stream_list, sender));
    }
    Ok(())
}

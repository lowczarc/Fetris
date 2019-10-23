use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

mod client_handler;
mod game;
mod network;

fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("0.0.0.0:3000")?;
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

use clap::{App, Arg};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

mod client_handler;
mod consts;
mod game;
mod network;

fn main() -> Result<(), std::io::Error> {
    let cli_matches = App::new("Fetris server")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("Port")
                .short("p")
                .long("port")
                .takes_value(true)
                .help(&format!(
                    "Port number to listen (default: {})",
                    consts::DEFAULT_PORT
                ))
                .value_name("PORT"),
        )
        .arg(
            Arg::with_name("Pool Size")
                .short("s")
                .long("pool-size")
                .takes_value(true)
                .help(&format!(
                    "The number of player in a pool (default: {})",
                    consts::DEFAULT_POOL_SIZE
                ))
                .value_name("SIZE"),
        )
        .get_matches();

    let listening_port = cli_matches
        .value_of("Port")
        .map_or(Ok(consts::DEFAULT_PORT), |p| p.parse())
        .unwrap_or_else(|_| panic!("Invalid Port number"));

    let pool_size = cli_matches
        .value_of("Pool Size")
        .map_or(Ok(consts::DEFAULT_POOL_SIZE), |p| p.parse())
        .unwrap_or_else(|_| panic!("Invalid Pool Size"));

    if pool_size < 1 {
        panic!("Invalid Pool Size");
    }

    let listener = TcpListener::bind(&format!("0.0.0.0:{}", listening_port))?;
    println!("Listening on port {}", listening_port);
    let stream_list = network::StreamList::new();
    let (sender, receiver) = mpsc::channel::<network::NetworkPacket>();

    {
        let stream_list = stream_list.clone();
        let options = game::Options {
            pool_size,
        };

        thread::spawn(move || game::game_main_thread(stream_list, receiver, options));
    }

    for stream in listener.incoming() {
        let stream_list = stream_list.clone();
        let sender = sender.clone();

        thread::spawn(move || client_handler::client_handler(stream.unwrap(), stream_list, sender));
    }
    Ok(())
}

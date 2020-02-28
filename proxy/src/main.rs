use fetris_protocol::{game::Input, ClientRequest, ServerRequest};
use serde_json;
use std::env;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread::spawn;
use tungstenite::protocol::{Role, WebSocket};
use tungstenite::server::accept;
use tungstenite::Message;

const DEFAULT_SERVER_PORT: &str = "3001";
const DEFAULT_PORT: &str = "9001";

const SERVER_ADDR: &'static str = "localhost";

fn main() {
    let server_port = if let Ok(server_port) = env::var("SERVER_PORT") {
        server_port
    } else {
        DEFAULT_SERVER_PORT.into()
    };

    let port = if let Ok(port) = env::var("SERVER_PORT") {
        port
    } else {
        DEFAULT_PORT.into()
    };

    let server = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    for stream in server.incoming() {
        let server_port = server_port.clone();
        spawn(move || {
            let stream_read = stream.unwrap();
            let stream_write = stream_read.try_clone().unwrap();
            let mut server_write =
                TcpStream::connect(format!("{}:{}", SERVER_ADDR, server_port)).unwrap();
            let server_read = server_write.try_clone().unwrap();
            let mut ws_read = accept(stream_read).unwrap();
            let mut ws_write = WebSocket::from_raw_socket(stream_write, Role::Server, None);

            spawn(move || loop {
                if let Ok(request) = ServerRequest::from_reader(&server_read) {
                    if let Err(_) = ws_write
                        .write_message(Message::Text(serde_json::to_string(&request).unwrap()))
                    {
                        println!("Server to websocket connection closed");
                        break;
                    }
                } else {
                    break;
                }
            });
            server_write
                .write(&ClientRequest::AskForAGame.into_bytes())
                .unwrap();
            loop {
                let msg = if let Ok(msg) = ws_read.read_message() {
                    msg
                } else {
                    println!("WebSocket to server connection closed");
                    break;
                };
                let msg = if let Message::Text(msg_string) = msg {
                    msg_string
                } else {
                    String::new()
                };
                let input = match msg.as_str() {
                    "Left" => Some(Input::Left),
                    "Right" => Some(Input::Right),
                    "Rotate" => Some(Input::Rotate),
                    "FastMove" => Some(Input::FastMove),
                    "Acceleration" => Some(Input::Acceleration),
                    "StockTetrimino" => Some(Input::StockTetrimino),
                    _ => None,
                };
                if let Some(input) = input {
                    println!("Input sent: {:?}", input);
                    server_write
                        .write(&ClientRequest::Input(input).into_bytes())
                        .unwrap();
                }
            }
        });
    }
}

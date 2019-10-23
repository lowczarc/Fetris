use fetris_protocol::{ClientRequest, ServerRequest};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::Receiver;

use crate::network::{NetworkAction, NetworkPacket, StreamList};

pub fn game_main_thread(stream_list: StreamList, receiver: Receiver<NetworkPacket>) {
    let mut players: HashMap<SocketAddr, ()> = HashMap::new();

    for packet in receiver.iter() {
        match packet.action {
            NetworkAction::OpenStream => {
                players.insert(packet.addr, ());
                println!("{} opened stream", packet.addr);
                let _ = stream_list.send_to(
                    &packet.addr,
                    ServerRequest::Message(
                        "Server".into(),
                        format!("Hello {}, welcome to the basic chat !", packet.addr),
                    ),
                );
            }
            NetworkAction::Request(ClientRequest::Message(message)) => {
                for (player, _) in players.iter() {
                    let _ = stream_list.send_to(
                        &player,
                        ServerRequest::Message(format!("{}", packet.addr), message.clone()),
                    );
                }
            }
            NetworkAction::CloseStream => {
                players.remove(&packet.addr);
                println!("{} closed stream", packet.addr);
            }
        }
    }
}

use fetris_protocol::ClientRequest;
use std::net::TcpStream;
use std::sync::mpsc::Sender;

use crate::network::{NetworkAction, NetworkPacket, StreamList};

pub fn client_handler(stream: TcpStream, stream_list: StreamList, sender: Sender<NetworkPacket>) {
    let addr = stream_list.open_stream(&stream).unwrap();
    sender
        .send(NetworkPacket::new(addr, NetworkAction::OpenStream))
        .unwrap();

    loop {
        if let Ok(request) = ClientRequest::from_reader(&stream) {
            sender
                .send(NetworkPacket::new(addr, NetworkAction::Request(request)))
                .unwrap();
        } else {
            break;
        }
    }

    sender
        .send(NetworkPacket::new(addr, NetworkAction::CloseStream))
        .unwrap();

    stream_list.close_stream(&addr);
}

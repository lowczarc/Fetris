use fetris_protocol::{ClientRequest, ServerRequest};
use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};

pub enum NetworkAction {
    OpenStream,
    Request(ClientRequest),
    CloseStream,
}

pub struct NetworkPacket {
    pub addr: SocketAddr,
    pub action: NetworkAction,
}

impl NetworkPacket {
    pub fn new(addr: SocketAddr, action: NetworkAction) -> Self {
        Self { addr, action }
    }
}

pub struct StreamList(Arc<Mutex<HashMap<SocketAddr, TcpStream>>>);

pub enum SendStreamError {
    UnknownAddr,
    CommunicationError(std::io::Error),
}

impl StreamList {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    pub fn open_stream(&self, stream: &TcpStream) -> Result<SocketAddr, std::io::Error> {
        let stream = stream.try_clone();

        if let Ok(stream) = stream {
            let addr = stream.peer_addr();

            if let Ok(addr) = addr {
                let mut all_streams = self.0.lock().unwrap();

                all_streams.insert(addr, stream);
                Ok(addr)
            } else {
                Err(addr.unwrap_err())
            }
        } else {
            Err(stream.unwrap_err())
        }
    }

    pub fn close_stream(&self, addr: &SocketAddr) {
        let mut all_streams = self.0.lock().unwrap();

        all_streams.remove(addr);
    }

    pub fn send_to(
        &self,
        addr: &SocketAddr,
        request: ServerRequest,
    ) -> Result<(), SendStreamError> {
        let all_streams = self.0.lock().unwrap();

        if let Some(mut stream) = all_streams.get(&addr) {
            if let Err(err) = stream.write(&request.into_bytes()) {
                return Err(SendStreamError::CommunicationError(err));
            }
            Ok(())
        } else {
            return Err(SendStreamError::UnknownAddr);
        }
    }
}

impl Clone for StreamList {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

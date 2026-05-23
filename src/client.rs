use std::net::TcpStream;
use std::time::Instant;

pub struct Client {
    pub socket: TcpStream,
    pub buffer: Vec<u8>,
    pub last_activity: Instant,
}

impl Client {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            buffer: Vec::new(),
            last_activity: Instant::now(),
        }
    }
}
use std::net::TcpStream;
use std::time::Instant;

pub struct Client {
    pub socket: TcpStream,
    pub buffer: Vec<u8>,
    pub port: u16,
    pub last_activity: Instant,
}

impl Client {
    pub fn new(socket: TcpStream, port: u16) -> Self {
        Self {
            socket,
            buffer: Vec::new(),
            port,
            last_activity: Instant::now(),
        }
    }
}
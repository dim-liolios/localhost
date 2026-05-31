use std::net::TcpStream;
use std::time::Instant;

pub struct Client {
    pub socket: TcpStream,
    pub buffer: Vec<u8>,
    pub port: u16,
    pub last_activity: Instant,
    pub cgi_waiting: bool,
    pub cgi_pid: Option<i32>,
    pub cgi_output_path: Option<String>,
}

impl Client {
    pub fn new(socket: TcpStream, port: u16) -> Self {
        Self {
            socket,
            buffer: Vec::new(),
            port,
            last_activity: Instant::now(),
            cgi_waiting: false,
            cgi_pid: None,
            cgi_output_path: None,
        }
    }
}
use std::net::TcpListener;

pub fn create_listener(addr: &str) -> TcpListener {
    let listener = TcpListener::bind(addr).expect("Failed to bind");

    listener.set_nonblocking(true).expect("Failed to set non-blocking");

    println!("Server listening on {}", addr);
    
    listener
}
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;
use std::io::Read;
use libc::{epoll_create1, epoll_ctl, epoll_wait, epoll_event, EPOLLIN, EPOLL_CTL_ADD, EPOLL_CTL_DEL};
// function, function, function, struct, constant, constant, constant

const MAX_EVENTS: usize = 64;

pub fn run(listener: TcpListener) {
    let listener_fd = listener.as_raw_fd();

    // 1. create epoll instance and get its file descriptor
    let epoll_fd = unsafe { epoll_create1(0) };
    if epoll_fd < 0 { 
        eprintln!("Failed to create epoll instance");
        return;
    }

    // 2. create event struct describing which fd to watch and for what events
    // we will use this struct for both the listener socket and client sockets
    let mut listener_event = epoll_event {
        events: EPOLLIN as u32,
        u64: listener_fd as u64,
    };

    // 3. add the listener_fd to the epoll instance
    let event_add_result = unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_ADD, listener_fd, &mut listener_event) };
    if event_add_result < 0 {
        eprintln!("Failed to add listener_fd to epoll instance");
        return;
    }

    // 4. create a buffer to hold events (epoll_event structs) returned by epoll_wait
    let mut events = vec![epoll_event { events: 0, u64: 0 }; MAX_EVENTS];

    // 5. create Client struct to hold clients (TcpStream) with a buffer (for inc data in chunks - larger than 4096 bytes)
    struct Client {
        socket: TcpStream,
        buffer: Vec<u8>,
    }
    
    // 6. create a vector to hold client connections (Client structs)
    let mut clients: Vec<Client> = Vec::new();

    println!("Event loop started");

    loop {
        // 7. wait for events on the epoll instance (blocking call, n = number of events returned)
        let n = unsafe {
            epoll_wait(epoll_fd, events.as_mut_ptr(), MAX_EVENTS as i32, -1)
        };
        if n < 0 {
            eprintln!("epoll_wait failed");
            break;
        }

        for i in 0..n as usize {
            let fd = events[i].u64 as i32;

            // fd = listener_fd => new client connecting --------------------------------------------------------------
            if fd == listener_fd {
                match listener.accept() {
                    Ok((client_socket, addr)) => {
                        println!("New connection from {}", addr);
                        client_socket.set_nonblocking(true).expect("Failed to set non-blocking");
                        let client_fd = client_socket.as_raw_fd();

                        let mut client_event = epoll_event {
                            events: EPOLLIN as u32,
                            u64: client_fd as u64,
                        };

                        // add the client_fd to the epoll instance like in 3. above for listener_fd
                        let client_add_result = unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_ADD, client_fd, &mut client_event) };
                        if client_add_result < 0 {
                            eprintln!("Failed to add client to epoll");
                        } else {
                            clients.push(Client { socket: client_socket, buffer: Vec::new() }); // important to keep the connection alive
                        }
                    }
                    Err(e) => eprintln!("accept() failed: {}", e),
                }
            } else {
                // fd = client_fd => existing client sent data --------------------------------------------------------
                if let Some(client) = clients.iter_mut().find(|c| c.socket.as_raw_fd() == fd) {
                // find() returns Option (Some(value) -> TcpStream or None)

                    let mut buffer = [0u8; 4096]; // array with 4096 8-bit integers -> bytes (0s here) to hold the data read from the client
                
                    match client.socket.read(&mut buffer) {
                    // read() returns Result (usize -> n bytes or Error)
                        Ok(0) => {
                            let client_remove_result = unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_DEL, fd, std::ptr::null_mut()) };
                            if client_remove_result < 0 {
                                eprintln!("Failed to remove client from epoll");
                            }
                            clients.retain(|c| c.socket.as_raw_fd() != fd); // remove dead client from clients vector anyway
                            println!("Client fd {} disconnected", fd);
                        }
                        Ok(bytes_read) => {
                            client.buffer.extend_from_slice(&buffer[..bytes_read]);
                            // extend_from_slice() appends the bytes read from the client to the client's buffer

                            while let Some(end) = check_request_end(&client.buffer) {
                                let request = String::from_utf8_lossy(&client.buffer[..end]).to_string();
                                
                                println!("Received request:\n{}", request);
                                client.buffer.drain(..end);
                            }
                        }
                        Err(e) => eprintln!("read() failed: {}", e),
                    }
                }
            }
        }
    }
} 


// ====================================================================================================================
// HELPER FUNCTIONS:

fn check_request_end(buffer: &[u8]) -> Option<usize> {
    let headers_end = buffer.windows(4).position(|w| w == b"\r\n\r\n")? + 4;
    let headers = std::str::from_utf8(&buffer[..headers_end]).ok()?;

    // case 1: chunked transfer encoding
    if headers.lines().any(|l| {
        let l = l.to_ascii_lowercase();
        l.starts_with("transfer-encoding:") && l.contains("chunked")
    }) {
        let body = &buffer[headers_end..];
        let end = body.windows(5).position(|w| w == b"0\r\n\r\n")?;
        return Some(headers_end + end + 5);
    }

    // case 2: POST with content-length
    if let Some(len) = extract_content_length(headers) {
        return if buffer.len() >= headers_end + len {
            Some(headers_end + len)
        } else {
            None // body not fully received yet
        };
    }

    // case 3: no body (GET, HEAD, DELETE, etc.)
    let method = headers.lines().next()?.split_whitespace().next()?;
    if matches!(method, "POST" | "PUT" | "PATCH") {
        return None;
    }

    Some(headers_end)
}

fn extract_content_length(headers: &str) -> Option<usize> {
    headers.lines().find_map(|line| {
        if line.to_ascii_lowercase().starts_with("content-length:") {
            line.split(':').nth(1)?.trim().parse().ok()
        } else {
            None
        }
    })
}

// ====================================================================================================================
// TESTS:

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_end_chunked() {
        let req = b"POST / HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\n\r\n12\r\nhello from chunked\r\n0\r\n\r\n";
        assert_eq!(check_request_end(req), Some(req.len()));
    }

    #[test]
    fn test_request_end_content_length() {
        let req = b"POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 5\r\n\r\nhello";
        assert_eq!(check_request_end(req), Some(req.len()));
    }

    #[test]
    fn test_request_end_get() {
        let req = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        assert_eq!(check_request_end(req), Some(req.len()));
    }

    #[test]
    fn test_request_end_incomplete_body() {
        let req = b"POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 100\r\n\r\nhello";
        assert_eq!(check_request_end(req), None);
    }
}

/* ====================================================================================================================
NOTES:

- epoll_create1 tells Kernel to create an epoll instance and returns an integer that is its file descriptor
    The 0 argument is for flags, which we set to 0 for default behavior

- unsafe: required to call libc syscalls like epoll_create1 bc they are not from Rust std library (cargo build will not run without this)

- if epoll_create1 returns a negative value, it indicates an error occurred while creating the epoll instance,
    otherwise it returns a non-negative file descriptor for the epoll instance

- event = epoll_event:
    we create an epoll_event struct to specify which listener socket we want to monitor (using its fd) and for what kind of events
    we set the events field to EPOLLIN (data available to read — on the listener socket this means a new connection is ready to accept())

- epoll_ctl: adds, modifies, or removes file descriptors from the kernel's interest list of the epoll instance
    we call epoll_ctl to add the listener socket to the epoll and later to add client sockets as well
    same as with epoll_create1, if it returns a negative value, it indicates an error occurred while adding the fd to the epoll instance

- EPOLLIN: constant from libc that indicates we want to be notified when there is data to read on the file descriptor
    if used on the listener socket, this means a new connection is ready to accept()
    if used on client sockets, this means there is data to read from the client
    EPOLLIN  = 0x00000001
    EPOLLOUT = 0x00000004
    they are bit flags, so you can combine them using bitwise OR (|) if you want to monitor multiple events on the same fd

- events vector:
    epoll_event is a C struct from libc, so we need to initialize it with default values (events: 0, u64: 0)
    if this wasn't the case we would use: vec![epoll_event::default(); MAX_EVENTS]

-  events.as_mut_ptr():
    we need a mutable pointer to the structs ("epoll_event") inside the "events" vector, not to the vector itself, because epoll_wait 
    will write the events that occurred directly into the memory locations of those structs
    (these structs have already their position in memory bc we created a vector with 64 empty structs)
    what we need: *mut epoll_event (we get it from events.as_mut_ptr())
    what we dont need: &mut events would give us a &mut Vec<epoll_event> (reference to the vector itself, not the structs inside it)

- clients vector:
    we need to store the TcpStream objects for the clients in a vector outside the loop so that they are not dropped at the end of
    each loop iteration due to how ownership works in Rust

- epoll loop:
    the first time we call epoll_wait we will get events only for the listener socket, (listener events = new client connection) bc its
    the only fd registered with the epoll instance at that point
    but after we accept a new client connection, we add its fd to the epoll instance, so that in the next iterations of the loop we will also
    get events for that client (client events = data sent)
    
- listener.accept():
    this TcpListener method returns a Result<(TcpStream, SocketAddr), std::io::Error>
    if successful, it gives us a TcpStream for communicating with the client and the client's SocketAddr (IP and port)
    if it fails, it gives us an error which we print to stderr

- client.socket.read(&mut buffer):
    reads up to 4096 bytes at a time from the kernel's TCP receive buffer; any remaining data stays in the kernel buffer
    this fd will be reported as readable again on the next epoll_wait call until all data has been consumed
    
- client.buffer.drain(..end):
    drain() removes only a complete request (even if it is its last part), leaving any remainging data from the next request
    in the buffer for the next loop iteration (thats why client.buffer.clear() would be wrong,
    because it would remove all data from the buffer, including any incomplete request that we haven't processed yet)

*/
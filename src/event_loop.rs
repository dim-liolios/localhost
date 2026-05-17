use std::net::{TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;
use libc::{epoll_create1, epoll_ctl, epoll_wait, epoll_event, EPOLLIN, EPOLL_CTL_ADD};
// function, function, function, struct, constant, constant

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
    let mut event = epoll_event {
        events: EPOLLIN as u32,
        u64: listener_fd as u64,
    };

    // 3. add the listener_fd to the epoll instance
    let event_add_result = unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_ADD, listener_fd, &mut event) };
    if event_add_result < 0 {
        eprintln!("Failed to add listener_fd to epoll instance");
        return;
    }

    // 4. create a buffer to hold events (epoll_event structs) returned by epoll_wait
    let mut events = vec![epoll_event { events: 0, u64: 0 }; MAX_EVENTS];

    // 5. create a vector to hold client connections (TcpStream objects) that we will accept in the event loop
    let mut clients: Vec<TcpStream> = Vec::new();

    println!("Event loop started");

    loop {
        // 6. wait for events on the epoll instance (blocking call, n = number of events returned)
        let n = unsafe {
            epoll_wait(epoll_fd, events.as_mut_ptr(), MAX_EVENTS as i32, -1)
        };
        if n < 0 {
            eprintln!("epoll_wait failed");
            break;
        }

        for i in 0..n as usize {
            let fd = events[i].u64 as i32;

            // 7. new client connecting
            if fd == listener_fd {
                match listener.accept() {
                    Ok((client_stream, addr)) => {
                        println!("New connection from {}", addr);
                        client_stream.set_nonblocking(true).expect("Failed to set non-blocking");
                        let client_fd = client_stream.as_raw_fd();

                        // register client with epoll so we get notified when they send data
                        let mut client_event = epoll_event {
                            events: EPOLLIN as u32,
                            u64: client_fd as u64,
                        };
                        let result = unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_ADD, client_fd, &mut client_event) };
                        if result < 0 {
                            eprintln!("Failed to add client to epoll");
                        } else {
                            clients.push(client_stream); // keep alive
                        }
                    }
                    Err(e) => eprintln!("accept() failed: {}", e),
                }
            } else {
                // 7. existing client sent data
                println!("Data from client fd: {}", fd);
            }
        }
    }
} 

/* ====================================================================================================================
// HELPER FUNCTIONS:




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

- epoll_ctl: adds, modifies, or removes file descriptors from the interest list of the epoll instance
    here we call epoll_ctl to add the listener socket to the epoll
    same as with epoll_create1, if it returns a negative value, it indicates an error occurred while adding the fd to the epoll instance

- EPOLLIN: constant from libc that indicates we want to be notified when there is data to read on the file descriptor
    if used on the listener socket, this means a new connection is ready to accept()
    if used on client sockets, this means there is data to read from the client
    EPOLLIN  = 0x00000001
    EPOLLOUT = 0x00000004
    they are bit flags, so you can combine them using bitwise OR (|) if you want to monitor multiple events on the same fd

- events vetor:
    epoll_event is a C struct from libc, so we need to initialize it with default values (events: 0, u64: 0)
    if this wasn't the case we would use: vec![epoll_event::default(); MAX_EVENTS]

-  events.as_mut_ptr():
    we need a mutable pointer to the structs ("epoll_event") inside the "events" vector, not to the vector itself, because epoll_wait 
    will write the events that occurred directly into the memory locations of those structs
    (these structs have already their position in memory bc we created a vector with 64 empty structs)
    what we need: *mut epoll_event (we get it from events.as_mut_ptr())
    what we dont need: &mut events would give us a &mut Vec<epoll_event> (reference to the vector itself, not the structs inside it)

*/
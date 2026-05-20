// TCP Socket accepts connections
#[test]
fn test_tcp_listener_binds() {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:8081");
    assert!(listener.is_ok(), "Failed to bind to port 8081");
}

// epoll event loop initiates
#[test]
fn test_epoll_creates_instance() {
    let epoll_fd = unsafe { libc::epoll_create1(0) };
    assert!(epoll_fd >= 0, "Failed to create epoll instance");
    unsafe { libc::close(epoll_fd) };
}

// epoll_ctl (EPOLL_CTL_ADD/DEL) works, i.e. listener socket registers successfully (and removed after) in event loop
#[test]
fn test_epoll_ctl_add_and_del() {
    use std::net::TcpListener;
    use std::os::unix::io::AsRawFd;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let listener_fd = listener.as_raw_fd();

    let epoll_fd = unsafe { libc::epoll_create1(0) };
    assert!(epoll_fd >= 0, "Failed to create epoll instance");

    let mut event = libc::epoll_event {
        events: libc::EPOLLIN as u32,
        u64: listener_fd as u64,
    };

    let add_result = unsafe { libc::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, listener_fd, &mut event) };
    assert_eq!(add_result, 0, "EPOLL_CTL_ADD failed");

    let del_result = unsafe { libc::epoll_ctl(epoll_fd, libc::EPOLL_CTL_DEL, listener_fd, std::ptr::null_mut()) };
    assert_eq!(del_result, 0, "EPOLL_CTL_DEL failed");

    unsafe { libc::close(epoll_fd) };
}
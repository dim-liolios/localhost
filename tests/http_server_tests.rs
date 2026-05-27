// TCP Socket accepts connections
#[test]
fn test_tcp_listener_binds() {
    use std::net::TcpListener;
    let _listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
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

// #[test]
// // #[ignore = "requires server running on localhost:8080"]
// fn curl_get_request() {
//     use std::process::Command;
//     let output = Command::new("curl")
//         .arg("-s") // silent mode to suppress progress output
//         .arg("-i") // include response headers in output
//         .arg("http://localhost:8080/")
//         .output()
//         .expect("Failed to execute curl command");
//     assert!(output.status.success(), "Curl command failed");
//     let response = String::from_utf8_lossy(&output.stdout);
//     assert!(response.contains("HTTP/1.1 200 OK"), "Expected 200 OK response, got:\n{}", response);
//     assert!(response.contains("<!DOCTYPE html>"), "Expected HTML content in response, got:\n{}", response);
// }   

// #[test]
// fn curl_post_request() {
//     use std::process::Command;
//     std::fs::write("test_upload.txt", "This is a test file for upload.").expect("Failed to create test upload file");

//     let output = Command::new("curl")
//         .arg("-s") // silent mode to suppress progress output
//         .arg("-i") // include response headers in output
//         .arg("-X")
//         .arg("POST")
//         .arg("-F")
//         .arg("file=@test_upload.txt") // Assuming test_upload.txt exists in the current directory
//         .arg("http://localhost:8080/uploads")
//         .output()
//         .expect("Failed to execute curl command");
//     assert!(output.status.success(), "Curl command failed");
//     let response = String::from_utf8_lossy(&output.stdout);
//     assert!(response.contains("HTTP/1.1 201 Created"), "Expected 201 Created response, got:\n{}", response);
//     assert!(response.contains("File 'test_upload.txt' uploaded successfully!"), "Expected success message in response, got:\n{}", response);
// }
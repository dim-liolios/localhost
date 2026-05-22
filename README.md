# localhost

A lightweight HTTP/1.1 web server written in Rust from scratch, built on top of Linux's `epoll` API for non-blocking, event-driven I/O — no async runtimes, no framework dependencies.

## Goal

The goal is a fully self-contained HTTP server capable of serving static web content, handling file uploads, managing client sessions, and executing CGI scripts — all within a single process and single thread using an epoll-driven event loop.

## How It Works

Incoming connections are managed through an `epoll` event loop. The server registers each client socket with epoll and processes read/write events as they become ready, keeping all I/O non-blocking. Request data is buffered per-client until a complete HTTP message is received, then parsed and dispatched to the appropriate handler.

## What Is Implemented

- **Non-blocking TCP listener** bound to a configurable address and port
- **epoll event loop** — single process, single thread, all I/O through epoll
- **HTTP/1.1 request parsing** — method, path, version, headers, and body
- **Chunked and unchunked request handling** — both transfer encodings are detected and buffered correctly
- **Request validation** — method whitelist, path traversal protection, version check, body size limit
- **GET handler** — serves static files from a configured root directory
- **POST handler** — accepts `multipart/form-data` file uploads and saves them to disk
- **HTTP response serialization** — correctly formatted status line, headers, and body for both success and error responses

## Stack

- **Language**: Rust
- **System calls**: `epoll_create1`, `epoll_ctl`, `epoll_wait` via the `libc` crate
- **Multipart parsing**: `multer` crate for handling file upload bodies

## Running

```bash
cargo run
```

Server starts on `127.0.0.1:8080` by default.

```bash
# Test GET
curl -i http://localhost:8080/

# Test file upload
curl -i -X POST -F "file=@photo.jpg" http://localhost:8080/uploads
```

## Testing

```bash
cargo test
```

Integration tests use `curl` and raw `libc` calls to verify the epoll setup and HTTP behavior end to end.

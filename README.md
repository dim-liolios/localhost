# localhost

HTTP/1.1 web server written in Rust with Linux `epoll`, built without async runtimes like tokio.

## Overview

This project serves static pages, handles uploads, supports route-level configuration, and runs Python CGI scripts, all from a single-process event loop.

## Implemented Features

- Rust server with non-blocking sockets and epoll-based multiplexing.
- HTTP request parsing for method, path, version, headers, and body.
- Supported methods: GET, POST, DELETE.
- Chunked and content-length request framing support.
- Route resolution by server (host/port) and longest matching route prefix.
- Route controls from config:
	- methods
	- root
	- index
	- directory_listing
	- redirect
	- cgi extension
	- cookie_required
- Static file serving and default index handling for directory targets.
- JSON directory listing endpoint support.
- Multipart file upload handling using `multer`.
- File deletion endpoint handling.
- Python CGI execution via `fork`/`execvp` with deferred response handling.
- Per-server body size limit enforcement (`client_max_body_size`).
- Custom error page lookup from config with template fallback.
- Request timeout handling (408) and idle client cleanup.

## Configuration

Config is read from `config/server.conf` and parsed by `src/config_parser`.

Supported directives:

- Server-level:
	- `host`
	- `ports`
	- `server_name`
	- `client_max_body_size`
	- `error_page <code> <path>`
- Route-level:
	- `methods`
	- `root`
	- `index`
	- `directory_listing on|off`
	- `redirect <code> <target>`
	- `cgi <extension>`
	- `cookie_required on|off`

Duplicate ports inside one `server {}` block are rejected by the parser.

## Current Web Pages

- `/` -> Homepage (`www/index.html`)
- `/uploads_entries` -> Uploads page (`www/uploads.html`)
- `/admin` -> Admin page (cookie-gated)
- Error responses render `www/error.html` by default, or configured error page files when present.

## Run

```bash
cargo run
```

## Quick Checks

```bash
# Homepage
curl -i http://127.0.0.1:8080/

# Upload
curl -i -X POST -F "file=@www/uploads/test.txt" http://127.0.0.1:8080/uploads

# List uploads
curl -i http://127.0.0.1:8080/uploads/list

# Delete uploaded file
curl -i -X DELETE http://127.0.0.1:8080/uploads/delete/test.txt

# Redirect route example
curl -i http://127.0.0.1:8080/redirect
```

## Test

```bash
cargo test
```

## Notes

- This project targets Linux/WSL for epoll and Unix fd APIs.
- Browser pages use same-origin API calls (relative URLs) to avoid CORS issues across host/port combinations.

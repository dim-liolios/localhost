use crate::http::request::HttpRequest;
use crate::http::handle_requests::RouteAction;
use crate::http::response::{ HttpResponseError , HttpResponseOk };
use crate::config::RouteConfig;
use crate::client::Client;
use std::fs::File;
use std::io::Write;
use bytes::Bytes;
use futures_util::stream;
use futures::executor::block_on;
use std::path::Path;
use std::os::unix::io::AsRawFd;
use std::ffi::CString;



//later will check if route has directory_listing true
//in configuration file to call this function
impl HttpRequest {

    //handle by using multer crate to parse the multipart/form-data and save the uploaded file to
    // the uploads directory, then return a success response
    pub fn handle_uploaded_file(&self, route: &RouteConfig) -> Vec<u8> {
        // 1. Extract the boundary from the Content-Type header
        let content_type = match self.headers.get("Content-Type") {
            Some(ct) => ct,
            None => return HttpResponseError::new_err_response(400, "Bad Request 2"),
        };

        // Parse out the boundary token string
        let boundary = match content_type.split("boundary=").nth(1) {
            Some(b) => b.trim().to_string(),
            None => return HttpResponseError::new_err_response(400, "Bad Request 3"),
        };

        // 2. Convert your raw bytes vector into a one-shot async stream for Multer
        let body_bytes = Bytes::copy_from_slice(&self.body);
        let stream = stream::once(async move { Ok::<_, std::io::Error>(body_bytes) });
        
        // Initialize Multer
        let mut multipart = multer::Multipart::new(stream, boundary);
        let mut saved_file_name = String::new();

        // 3. Execute the async state machine blockingly on the current epoll thread
        block_on(async {
            loop {
                match multipart.next_field().await {
                    Ok(Some(mut field)) => {
                        if let Some(file_name) = field.file_name() {
                            // Prevent path traversal security vulnerabilities (e.g. "../../etc/passwd")
                            let safe_name = match Path::new(file_name).file_name() {
                                Some(n) => n,
                                None => return,
                            };
                            saved_file_name = safe_name.to_string_lossy().into_owned();

                            let save_path = Path::new(&route.root).join(&saved_file_name);

                            let mut file = match File::create(save_path) {
                                Ok(f) => f,
                                Err(_) => return,
                            };

                            loop {
                                match field.chunk().await {
                                    Ok(Some(chunk)) => {
                                        if file.write_all(&chunk).is_err() {
                                            return;
                                        }
                                    }
                                    Ok(None) => break,
                                    Err(_) => return,
                                }
                            }
                            let _ = file.flush();
                        }
                    }
                    Ok(None) => break,
                    Err(_) => return,
                }
            }
        });

        // 4. Return custom response body bytes signaling success
        if !saved_file_name.is_empty() {
            let response_body = format!("File '{}' uploaded successfully!", saved_file_name);
            HttpResponseOk {
                status_code: 201,
                headers: std::collections::HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]),
                body: response_body.into_bytes(),
            }.response_ok_to_bytes()
        } else {
            HttpResponseError::new_err_response(400, "No file uploaded")
        }
    }


    pub fn list_directory(&self, route: &RouteConfig) -> Vec<u8> {
        let mut entries = Vec::new();
        if let Ok(dir_entries) = std::fs::read_dir(route.root.clone()) {
            for entry in dir_entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    entries.push(file_name);
                }else{
                    entries.push("Invalid UTF-8 filename".to_string());
                }
            }
        }else{
            return HttpResponseError::new_err_response(404, "Directory Not Found");
        }
        println!("Directory listing: {:?}", entries);
        HttpResponseOk {
            status_code: 200,
            headers: std::collections::HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            body: serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string()).into_bytes(),
        }.response_ok_to_bytes()
    }


    pub fn handle_cgi(&self, path: &str, route: &RouteConfig, client: &mut Client) -> RouteAction {
        if client.cgi_waiting {
            return RouteAction::Deferred;
        }

        let script_rel = path
            .strip_prefix(&route.path)
            .unwrap_or(path)
            .trim_start_matches('/');
        let cgi_script_path = Path::new(&route.root).join(script_rel);

        if !cgi_script_path.exists() {
            return RouteAction::Immediate(HttpResponseError::new_err_response(404, "Not Found"));
        }

        let output_path = format!("/tmp/localhost-cgi-{}.out", client.socket.as_raw_fd());
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            return RouteAction::Immediate(HttpResponseError::new_err_response(500, "Internal Server Error"));
        }

        if pid == 0 {
            unsafe {
                let output_path_c = match CString::new(output_path.clone()) {
                    Ok(v) => v,
                    Err(_) => libc::_exit(1),
                };

                let fd = libc::open(
                    output_path_c.as_ptr(),
                    libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC,
                    0o644,
                );
                if fd < 0 {
                    libc::_exit(1);
                }

                libc::dup2(fd, libc::STDOUT_FILENO);
                libc::dup2(fd, libc::STDERR_FILENO);
                libc::close(fd);

                let program = match CString::new("python3") {
                    Ok(v) => v,
                    Err(_) => libc::_exit(1),
                };
                let script = match CString::new(cgi_script_path.to_string_lossy().to_string()) {
                    Ok(v) => v,
                    Err(_) => libc::_exit(1),
                };

                let argv = [program.as_ptr(), script.as_ptr(), std::ptr::null()];
                libc::execvp(program.as_ptr(), argv.as_ptr());
                libc::_exit(1);
            }
        }

        client.cgi_waiting = true;
        client.cgi_pid = Some(pid);
        client.cgi_output_path = Some(output_path);

        // DO NOT block
        RouteAction::Deferred
    }
}
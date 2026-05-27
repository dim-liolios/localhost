use crate::http::helpers::list_directory;
use crate::http::request::HttpRequest;
use crate::http::response::{ HttpResponseError, HttpResponseOk };
use std::fs::File;
use std::io::Write;
use std::path::Path;
use bytes::Bytes;
use futures_util::stream;
use futures::executor::block_on;
use crate::config::{RouteConfig, Method};



// pub struct ServerConfig {
//     pub host: IpAddr,                        // IP to listen on: "127.0.0.1"
//     pub ports: Vec<u16>,
//     pub server_name: String,                 // HTTP Host header (we can have multiple servers on same IP:port with different server_name)
//     pub error_pages: HashMap<u16, String>,   // 404: "./errors/404.html"
//     pub client_max_body_size: usize,
//     pub routes: Vec<RouteConfig>,
//     pub default_route: RouteConfig,          // if no route matches, use this (else 404)
// }

// // ====================================================================================================================

// pub struct RouteConfig {
//     pub path: String,
//     pub methods: Vec<Method>,
//     pub root: String,                        // directory to serve files from: "./www"
//     pub index_file: Option<String>,          // if "GET /about/" serve: "about/index.html" (else 404 or files if directory listing enabled)
//     pub directory_listing: bool,           
//     pub redirect: Option<(u16, String)>,     // redirect: (status code, target URL)
//     pub cgi_extension: Option<String>,
// }


impl HttpRequest {

    // handle GET request
    pub fn handle_get(&self, route: &RouteConfig) -> Vec<u8> {
        let index = route.index_file.as_deref().unwrap_or("index.html");
        let file_path = format!("{}/{}", route.root, index);
        match std::fs::read(&file_path) {
            Ok(body) => {
                let mut headers = std::collections::HashMap::new();
                headers.insert("Content-Type".to_string(), "text/html".to_string());
                HttpResponseOk { status_code: 200, headers, body }.response_ok_to_bytes()
            }
            _ => HttpResponseError::new_err_response(404, "Not Found"),
        }
    }
}


impl HttpRequest {
    // handle POST request
    pub fn handle_post(&self, route: &RouteConfig) -> Vec<u8> {
        if let Some(content_type) = self.headers.get("Content-Type") {
            if content_type.contains("multipart/form-data") {
                return self.handle_uploaded_file(route);
            }
        }
        HttpResponseError::new_err_response(400, "Bad Request")
    }

    //handle by using multer crate to parse the multipart/form-data and save the uploaded file to
    // the uploads directory, then return a success response
    fn handle_uploaded_file(&self, route: &RouteConfig) -> Vec<u8> {
        // 1. Extract the boundary from the Content-Type header
        let content_type = match self.headers.get("Content-Type") {
            Some(ct) => ct,
            None => return HttpResponseError::new_err_response(400, "Bad Request"),
        };

        // Parse out the boundary token string
        let boundary = match content_type.split("boundary=").nth(1) {
            Some(b) => b.trim().to_string(),
            None => return HttpResponseError::new_err_response(400, "Bad Request"),
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
}

//implementation for delete request
impl HttpRequest {
    pub fn handle_delete(&self) -> Vec<u8> {
        if !self.path.starts_with("/uploads/delete/") {
            return HttpResponseError::new_err_response(404, "Not Found");
        }
        let file_name = self.path.split("/uploads/delete/").nth(1).unwrap_or("");
        if file_name.is_empty() {
            return HttpResponseError::new_err_response(400, "Bad Request");
        }
        let file_path = Path::new("./www/uploads").join(file_name);
        if !file_path.exists() {
            return HttpResponseError::new_err_response(404, "File Not Found");
        }else{
            match std::fs::remove_file(file_path) {
                Ok(_) => {
                    HttpResponseOk {
                        status_code: 204,
                        headers: std::collections::HashMap::new(),
                        body: Vec::new(),
                    }.response_ok_to_bytes()
                }
                Err(_) => HttpResponseError::new_err_response(500, "Internal Server Error"),
            }
        }

    }
}




impl HttpRequest {
    // execute the route handler based on the request method and the route configuration, return the response bytes
    pub fn execute_route(&self, route: &RouteConfig) -> Vec<u8> {
        // check if method is allowed for this route
        let allowed = route.methods.iter().any(|m| match m {
            Method::GET => self.method == "GET",
            Method::POST => self.method == "POST",
            Method::DELETE => self.method == "DELETE",
        });
        if !allowed {
            return HttpResponseError::new_err_response(405, "Method Not Allowed");
        }
        match self.method.as_str() {
            "GET" => self.handle_get(route),
            "POST" => self.handle_post(route),
            _ => HttpResponseError::new_err_response(405, "Method Not Allowed"),
        }
    }

}




/* ====================================================================================================================
NOTES:

- in handle_get():
    first line: ckeck if the route has an index file specified, if not default to "index.html"
    second line: construct the full file path by combining the route's root directory with the index file name
    third line: attempt to read the file from disk, if successful create a 200 OK response with the file contents as the body
                if reading fails (e.g. file not found), return a 500 Internal Server Error response

*/

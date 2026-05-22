use crate::http::request::HttpRequest;
use crate::http::response::{ HttpResponseError, HttpResponseOk };
use std::fs::File;
use std::io::Write;
use std::path::Path;
use bytes::Bytes;
use futures_util::stream;
use futures::executor::block_on;

//Implementation for get - serve static files requests
impl HttpRequest {
    //simple implementation of get for testing
    //serving static index.html at path "/"
    //later we'll search register routes from config file
    pub fn handle_get(&self) -> Vec<u8> {
        if self.path != "/" {
            return HttpResponseError::new_err_response(404, "Not Found");
        }
        let body = std::fs::read("./routes/www/index.html").unwrap_or_else(|_| HttpResponseError::new_err_response(500, "Internal Server Error"));

        HttpResponseOk {
            status_code: 200,
            headers: std::collections::HashMap::new(),
            body,
        }.response_ok_to_bytes()
    }
}

//Iplementation for post - upload files requests
impl HttpRequest {
    pub fn handle_post(&self) -> Vec<u8> {
        if self.method != "POST" {
            return HttpResponseError::new_err_response(405, "Method Not Allowed");
        }
        if self.path != "/uploads" {
            return HttpResponseError::new_err_response(404, "Not Found");
        }
        if let Some(content_type) = self.headers.get("Content-Type") {
            if content_type.contains("multipart/form-data") {
                return self.handle_uploaded_file();
            }
        }
        HttpResponseError::new_err_response(400, "Bad Request")
    }
    //handle by using multer crate to parse the multipart/form-data and save the uploaded file to
    // the uploads directory, then return a success response
    fn handle_uploaded_file(&self) -> Vec<u8> {
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

                            let save_path = Path::new("./routes/uploads").join(&saved_file_name);

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


//Implementation for delete - delete files requests
impl HttpRequest {
// //TODO
// fn handle_delete(&self) -> Vec<u8> {
// }
}

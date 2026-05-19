
use std::collections::HashMap;

pub struct HttpResponseOk {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub struct HttpResponseError {
    pub status_code: u16,
    pub message: String,
}

impl HttpResponseError {
    //This will be served by html file for the error page later
    pub fn new_err_response(status_code: u16, body: &str) -> Vec<u8> {
        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
            status_code,
            body,
            body.len(),
            body
        );
        response.into_bytes()
    }
}


impl HttpResponseOk {
      pub fn into_bytes(self) -> Vec<u8> {
        let mut response = format!(
            "HTTP/1.1 {} OK\r\n",
            self.status_code
        );
        for (key, value) in self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        response.push_str(&format!("Content-Length: {}\r\n\r\n", self.body.len()));
        let mut response_bytes = response.into_bytes();
        response_bytes.extend_from_slice(&self.body);
        response_bytes
    }
}
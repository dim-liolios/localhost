
use std::collections::HashMap;
use crate::ERROR_TEMPLATE;

pub struct HttpResponseOk {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub struct HttpResponseError {
}

impl HttpResponseError {

    //Error.html is served for all error responses
    pub fn new_err_response(status_code: u16, status_text: &str) -> Vec<u8> {
        let body = ERROR_TEMPLATE
        .get()
        .unwrap()
        .replace("{{status_code}}", &status_code.to_string())
        .replace("{{status_text}}", status_text);
        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
            status_code,
            status_text,
            body.as_bytes().len(),
            body
        );
        response.into_bytes()
    }
}


impl HttpResponseOk {
      pub fn response_ok_to_bytes(self) -> Vec<u8> {
        let mut response = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status_code,
            match self.status_code {
                200 => "OK",
                201 => "Created",
                _ => "Unknown",
            }
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
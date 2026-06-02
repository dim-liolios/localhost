use crate::ERROR_TEMPLATE;
use std::collections::HashMap;

pub struct HttpResponseOk {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub struct HttpResponseError {
}

impl HttpResponseError {
    //This will be served by html file for the error page later
    pub fn new_err_response(status_code: u16, body: &str) -> Vec<u8> {

        let extra_html = if status_code == 403 {
            "<p><a href=\"/cgi/set_cookie.py\"><button type=\"button\">Get Cookie</button></a></p>"
        } else {
            ""
        };

        let html_body = ERROR_TEMPLATE
        .get()
        .unwrap()
        .replace("{{status_code}}", &status_code.to_string())
        .replace("{{status_text}}", body)
        .replace("{{extra_html}}", extra_html);
    
        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
            status_code,
            body,
            html_body.len(),
            html_body
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
                204 => "No Content",
                _ => "Unknown",
            }
        );
        if self.status_code == 204 {
            // No Content responses should not have a body
            //+blank line after headers to signal end of headers
             response.push_str("\r\n"); 
             return response.into_bytes();
        }
        for (key, value) in self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        response.push_str(&format!("Content-Length: {}\r\n\r\n", self.body.len()));
        let mut response_bytes = response.into_bytes();
        response_bytes.extend_from_slice(&self.body);
        response_bytes
    }
}
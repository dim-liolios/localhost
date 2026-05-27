use std::collections::HashMap;
use crate::http::response::HttpResponseError;

pub struct HttpRequest {
    pub method: String,
    pub path: String,
    version: String,
    pub headers: HashMap<String, String>,
    pub cookies: bool,
    pub body: Vec<u8>,
}



impl HttpRequest {
    
    pub fn parse_request(buffer: &[u8]) -> Result<HttpRequest, Vec<u8>> {

        //split the buffer to
        //1.request line + headers
        //2.body
        let header_end_index = match buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            Some(i) => i,
            None => return Err(HttpResponseError::new_err_response(400, "Bad Request")),
        };
        let header_part = &buffer[..header_end_index];
        let body_part = &buffer[header_end_index + 4..];

        let header_str = String::from_utf8_lossy(header_part);
        let mut lines = header_str.lines();


        //parse request line
        let request_line = match lines.next() {
            Some(line) => line,
            None => return Err(HttpResponseError::new_err_response(400, "Bad Request")),
        };
        let mut request_line_parts = request_line.split_whitespace();
        let method = match request_line_parts.next() {
            Some(m) => m.to_string(),
            None => return Err(HttpResponseError::new_err_response(400, "Bad Request")),
        };
        let path = match request_line_parts.next() {
            Some(p) => p.to_string(),
            None => return Err(HttpResponseError::new_err_response(400, "Bad Request")),
        };
       
        
        let version = match request_line_parts.next() {
            Some(v) => v.to_string(),
            None => return Err(HttpResponseError::new_err_response(400, "Bad Request")),
        };

        //so now we start from the second line of header - line 1
        let mut headers = HashMap::new();
        let mut cookie_exist = false;
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                if line.starts_with("Cookie:") {
                    cookie_exist = true;
                }
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        // reject oversized bodies before constructing the request
        if let Some(content_length) = headers.get("Content-Length") {
            if let Ok(length) = content_length.parse::<usize>() {
                if length > 10 * 1024 * 1024 {
                    return Err(HttpResponseError::new_err_response(413, "Payload Too Large"));
                }
            }
        }

        let request = HttpRequest {
            method,
            path,
            version,
            headers,
            cookies: cookie_exist,
            body: body_part.to_vec(),
        };
        //validate the request
        request.is_valid()
    }


    //return the response directly from the is_valid function, if the request is valid then 
    //we call handle_request to get the response, otherwise we return the error response directly
    fn is_valid(self) -> Result<HttpRequest, Vec<u8>> {
        //check method
        let valid_methods = ["GET", "POST", "DELETE"];
        if !valid_methods.contains(&self.method.as_str()) {
            return Err(HttpResponseError::new_err_response(405, "Method Not Allowed"));
        }

        //check path
        if self.path.is_empty() || !self.path.starts_with('/') || self.path.contains("..") {
            return Err(HttpResponseError::new_err_response(400, "Bad Request"));
        }

        //check version
        if self.version != "HTTP/1.1" && self.version != "HTTP/1.0" {
            return Err(HttpResponseError::new_err_response(400, "Bad Request"));
        }

        //check cookies for authentication - reminder to check requirements
        //Cookie check disabled for now
        // if self.headers.get("Cookie").is_none() {
        //     return Err(HttpResponseError::new_err_response(401, "Unauthorized"));
        // }

        //maybe add more checks here

        Ok(self)
    }
}



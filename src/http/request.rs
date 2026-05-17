//parse ready to read request from
//epoll wait


// struct HttpRequest {
//     method: String,
//     path: String,
//     version: String,
//     headers: HashMap<String, String>,
//     body: Vec<u8>,
// }

impl HttpRequest {
    
    fn parse_request(buffer: &[u8]) -> HttpResponse {

        //split the buffer to
        //1.request line + headers
        //2.body
        let header_end_index = buffer.windows(4).position(|window| window == b"\r\n\r\n")?;
        let header_part = &buffer[..header_end_index];
        let body_part = &buffer[header_end_index + 4..];

        let header_str = String::from_utf8_lossy(header_part);
        let mut lines = header_str.lines();


        //parse request line
        let request_line = lines.next()?;//with next line 0 is consumed
        let mut request_line_parts = request_line.split_whitespace();
        let method = request_line_parts.next()?.to_string();
        let path = request_line_parts.next()?.to_string();
       
        
        let version = request_line_parts.next()?.to_string();

        //so now we start from the second line of header - line 1
        let mut headers = HashMap::new();
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
                //
                //todo check content length not too big
                //avoid endless body attack
                //and probably handle the siege test
                
            }
        }

        let request = HttpRequest {
            method,
            path,
            version,
            headers,
            body: body_part.to_vec(),
        };
        //validate the request
        request.is_valid()
    }
    //change of tactic, instead of struct instance we 
    //return the response directly from the is_valid function, if the request is valid then 
    //we call handle_request to get the response, otherwise we return the error response directly
    fn is_valid(&self) -> Option<HttpResponse> {
        //check method
        let valid_methods = ["GET", "POST", "DELETE"];
        if !valid_methods.contains(&self.method.as_str()) {
            return HttpResponse::new(405, "Method Not Allowed");
        }

        //check path
        if self.path.is_empty() || !self.path.starts_with('/') || self.path.contains("..") {
            return HttpResponse::new(400, "Bad Request");
        }

        //check version
        if self.version != "HTTP/1.1" && self.version != "HTTP/1.0" {
            return HttpResponse::new(400, "Bad Request");
        }

        //check content length
        if let Some(content_length) = self.headers.get("Content-Length") {
            if let Ok(length) = content_length.parse::<usize>() {
                if length > 10 * 1024 * 1024 { //10MB limit
                    return HttpResponse::new(413, "Payload Too Large");
                }
            }
        }
        //could add more check here

        return handle_request(&self);
    }
    fn handle_request(&self) -> HttpResponse {
        
        match self.method.as_str() {
            "GET" => self.handle_get(),
            "POST" => self.handle_post(),
            "DELETE" => self.handle_delete(),
            _ => HttpResponse::new(405, "Method Not Allowed"),
        }
    }
    

}
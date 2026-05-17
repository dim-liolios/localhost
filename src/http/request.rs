
//parse ready to read request from
//epoll wait

struct HttpRequest {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpRequest {
    
    fn parse_request(buffer: &[u8]) -> Option<HttpRequest> {

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
        //security check for path
        if path.contains("..") {
            return None;
        }
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

        Some(HttpRequest {
            method,
            path,
            version,
            headers,
            body: body_part.to_vec(),
        })
    }

}
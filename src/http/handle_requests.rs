use crate::http::request::HttpRequest;
use crate::http::response::{ HttpResponseError, HttpResponseOk };


impl HttpRequest {
    //simple implementation of get for testing
    //serving static index.html at path "/"
    pub fn handle_get(&self) -> Vec<u8> {
        if self.path != "/" {
            return HttpResponseError::new_err_response(400, "Bad Request");
        }
        let body = std::fs::read("./routes/www/index.html").unwrap_or_else(|_| HttpResponseError::new_err_response(500, "Internal Server Error"));

        HttpResponseOk {
            status_code: 200,
            headers: self.headers.clone(),
            body,
        }.to_bytes()
    }

    // //TODO   
    // fn handle_post(&self) -> HttpResponse {
    // }
    // fn handle_delete(&self) -> HttpResponse {
    // }
}
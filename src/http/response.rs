

struct HttpResponse {
    status_code: u16,
    body: Vec<u8>,
}

impl HttpResponse {
    fn new(status_code: u16, body: &str) -> Self {
        HttpResponse {
            status_code,

            body: body.as_bytes().to_vec(),
        }
    }
}




//response struct for !200
//fn return the response with the code
//and simple htmkl body for the error page


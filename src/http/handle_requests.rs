use crate::http::request::HttpRequest;
use crate::http::response::{ HttpResponseError, HttpResponseOk };
use crate::config::{RouteConfig, Method};
use crate::client::Client;
use std::path::Path;

pub enum RouteAction {
    Immediate(Vec<u8>),
    Deferred,
}



//    route / {
//         methods GET
//         root ./www
//         index index.html
//         directory_listing off
//     }
//     test route
//     route /admin {
//         methods GET POST
//         root ./www
//         index admin.html
//         cookie_required true
//         directory_listing off
//     }

//     route /uploads { 
//         methods GET POST DELETE
//         root ./www/uploads
//         directory_listing on
//     }


impl HttpRequest {

    // handle GET request
    pub fn handle_get(&self, route: &RouteConfig, client: &mut Client) -> RouteAction {
        if route.cookie_required && !self.cookies {
            let body = b"<!DOCTYPE html><html><head><title>403 Forbidden</title></head><body><h1>403 Forbidden</h1><p>You need a cookie to access /admin.</p><p><a href=\"/cgi/set_cookie.py\">Get Cookie</a></p><p><a href=\"/admin\">Retry /admin</a></p></body></html>";
            let response = format!(
                "HTTP/1.1 403 Forbidden\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let mut bytes = response.into_bytes();
            bytes.extend_from_slice(body);
            return RouteAction::Immediate(bytes);
        }

        if self.path.ends_with(".py") {
            return self.handle_cgi(&self.path, route, client);
        }
        if route.directory_listing == true {
            return RouteAction::Immediate(self.list_directory());
        } else {
            let file_path = route.root.clone() + "/" + route.index_file.as_deref().unwrap().trim_start_matches('/');
            match std::fs::read(file_path) {
                Ok(contents) => RouteAction::Immediate(HttpResponseOk {
                    status_code: 200,
                    headers: std::collections::HashMap::from([("Content-Type".to_string(), "text/html".to_string())]),
                    body: contents,
                }.response_ok_to_bytes()),
                Err(_) => RouteAction::Immediate(HttpResponseError::new_err_response(404, "Not Found")),
            }
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
    pub fn execute_route(&self, route: &RouteConfig, client: &mut crate::client::Client) -> RouteAction {
        // check if method is allowed for this route
        let allowed = route.methods.iter().any(|m| match m {
            Method::GET => self.method == "GET",
            Method::POST => self.method == "POST",
            Method::DELETE => self.method == "DELETE",
        });
        if !allowed {
            return RouteAction::Immediate(HttpResponseError::new_err_response(405, "Method Not Allowed"));
        }
        match self.method.as_str() {
            "GET" => self.handle_get(route, client),
            "POST" => RouteAction::Immediate(self.handle_post(route)),
            _ => RouteAction::Immediate(HttpResponseError::new_err_response(405, "Method Not Allowed")),
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

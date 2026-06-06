use crate::http::request::HttpRequest;
use crate::http::response::{ HttpResponseError, HttpResponseOk };
use crate::config::{RouteConfig, Method, ServerConfig};
use crate::client::Client;
use std::path::{Path, PathBuf};

pub enum RouteAction {
    Immediate(Vec<u8>),
    Deferred,
}



impl HttpRequest {

    fn relative_route_path<'a>(&'a self, route: &RouteConfig) -> &'a str {
        if route.path == "/" {
            return self.path.trim_start_matches('/');
        }
        self.path
            .strip_prefix(&route.path)
            .unwrap_or("")
            .trim_start_matches('/')
    }

    fn resolve_get_file_path(&self, route: &RouteConfig, server: &ServerConfig) -> Result<PathBuf, Vec<u8>> {
        let rel = self.relative_route_path(route);
        let mut full_path = Path::new(&route.root).to_path_buf();

        if rel.is_empty() {
            if route.directory_listing {
                return Err(self.list_directory(route));
            }

            let index_file = match route.index_file.as_deref() {
                Some(index) => index,
                None => return Err(HttpResponseError::new_err_response_with_pages(404, "Not Found", &server.error_pages)),
            };
            full_path.push(index_file.trim_start_matches('/'));
            return Ok(full_path);
        }

        full_path.push(rel);
        if full_path.is_dir() {
            if route.directory_listing {
                return Err(self.list_directory(route));
            }

            let index_file = match route.index_file.as_deref() {
                Some(index) => index,
                None => return Err(HttpResponseError::new_err_response_with_pages(404, "Not Found", &server.error_pages)),
            };
            full_path.push(index_file.trim_start_matches('/'));
        }

        Ok(full_path)
    }

    // handle GET request
    pub fn handle_get(&self, route: &RouteConfig, server: &ServerConfig, client: &mut Client) -> RouteAction {
        if route.cookie_required && !self.cookies {
            return RouteAction::Immediate(HttpResponseError::new_err_response_with_pages(403, "Forbidden", &server.error_pages));
        }

        if let Some(ext) = route.cgi_extension.as_deref() {
            if self.path.ends_with(ext) {
                return self.handle_cgi(&self.path, route, client);
            }
        }

        let file_path = match self.resolve_get_file_path(route, server) {
            Ok(path) => path,
            Err(response) => return RouteAction::Immediate(response),
        };

        match std::fs::read(file_path) {
            Ok(contents) => RouteAction::Immediate(HttpResponseOk {
                status_code: 200,
                headers: std::collections::HashMap::from([("Content-Type".to_string(), "text/html".to_string())]),
                body: contents,
            }.response_ok_to_bytes()),
            Err(_) => RouteAction::Immediate(HttpResponseError::new_err_response_with_pages(404, "Not Found", &server.error_pages)),
        }
    }
}


impl HttpRequest {
    // handle POST request
    pub fn handle_post(&self, route: &RouteConfig, server: &ServerConfig) -> Vec<u8> {
        if let Some(content_type) = self.headers.get("Content-Type") {
            if content_type.contains("multipart/form-data") {
                return self.handle_uploaded_file(route);
            }
        }
        HttpResponseError::new_err_response_with_pages(400, "Bad Request", &server.error_pages)
    }
}

//implementation for delete request
impl HttpRequest {
    pub fn handle_delete(&self, route: &RouteConfig, server: &ServerConfig) -> Vec<u8> {
        let expected_prefix = format!("{}/", route.path.trim_end_matches('/'));
        if !self.path.starts_with(&expected_prefix) {
            return HttpResponseError::new_err_response_with_pages(404, "Not Found", &server.error_pages);
        }
        let file_name = self.path.strip_prefix(&expected_prefix).unwrap_or("");
        if file_name.is_empty() {
            return HttpResponseError::new_err_response_with_pages(400, "Bad Request", &server.error_pages);
        }
        let file_path = Path::new(&route.root).join(file_name);
        if !file_path.exists() {
            return HttpResponseError::new_err_response_with_pages(404, "File Not Found", &server.error_pages);
        }else{
            match std::fs::remove_file(file_path) {
                Ok(_) => {
                    HttpResponseOk {
                        status_code: 204,
                        headers: std::collections::HashMap::new(),
                        body: Vec::new(),
                    }.response_ok_to_bytes()
                }
                Err(_) => HttpResponseError::new_err_response_with_pages(500, "Internal Server Error", &server.error_pages),
            }
        }

    }
}




impl HttpRequest {
    // execute the route handler based on the request method and the route configuration, return the response bytes
    pub fn execute_route(&self, route: &RouteConfig, server: &ServerConfig, client: &mut crate::client::Client) -> RouteAction {
        if let Some((status, target)) = &route.redirect {
            let reason = match *status {
                301 => "Moved Permanently",
                302 => "Found",
                _ => "Found",
            };
            let response = format!(
                "HTTP/1.1 {} {}\r\nLocation: {}\r\nContent-Length: 0\r\n\r\n",
                status,
                reason,
                target
            );
            return RouteAction::Immediate(response.into_bytes());
        }
        // check if method is allowed for this route
        let allowed = route.methods.iter().any(|m| match m {
            Method::GET => self.method == "GET",
            Method::POST => self.method == "POST",
            Method::DELETE => self.method == "DELETE",
        });
        if !allowed {
            return RouteAction::Immediate(HttpResponseError::new_err_response_with_pages(405, "Method Not Allowed", &server.error_pages));
        }
        match self.method.as_str() {
            "GET" => self.handle_get(route, server, client),
            "POST" => RouteAction::Immediate(self.handle_post(route, server)),
            "DELETE" => RouteAction::Immediate(self.handle_delete(route, server)),
            _ => RouteAction::Immediate(HttpResponseError::new_err_response_with_pages(405, "Method Not Allowed", &server.error_pages)),
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

use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug)]
pub struct AppConfig {
    pub servers: Vec<ServerConfig>,        // one or more server blocks
}

// ====================================================================================================================

#[derive(Debug)]
pub struct ServerConfig {
    pub host: IpAddr,                        // IP to listen on: "127.0.0.1"
    pub ports: Vec<u16>,
    pub server_name: String,                 // HTTP Host header (we can have multiple servers on same IP:port with different server_name)
    pub error_pages: HashMap<u16, String>,   // 404: "./errors/404.html"
    pub client_max_body_size: usize,
    pub routes: Vec<RouteConfig>,
}

// ====================================================================================================================

#[derive(Debug)]
pub struct RouteConfig {
    pub path: String,
    pub methods: Vec<Method>,
    pub root: String,                        // directory to serve files from: "./www"
    pub index_file: Option<String>,          // if "GET /about/" serve: "about/index.html" (else 404 or files if directory listing enabled)
    pub directory_listing: bool,           
    pub redirect: Option<(u16, String)>,     // redirect: (status code, target URL)
    pub cgi_extension: Option<String>,
    pub cookie_required: bool,
}

// ====================================================================================================================

#[derive(Debug)]
pub enum Method {
    GET,
    POST,
    DELETE,
}

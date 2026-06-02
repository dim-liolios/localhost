mod server;
mod event_loop;
mod http;
mod client;
mod router;
mod config;
mod config_parser;
use std::collections::HashSet;
use crate::config_parser::parse_config_file;
use crate::config::{AppConfig, Method, RouteConfig, ServerConfig};
use std::sync::OnceLock;
use std::collections::HashMap;

pub static ERROR_TEMPLATE: OnceLock<String> = OnceLock::new();

fn hardcoded_test_config() -> AppConfig {
    let routes = vec![
        RouteConfig {
            path: "/".to_string(),
            methods: vec![Method::GET],
            root: "./www".to_string(),
            index_file: Some("index.html".to_string()),
            directory_listing: false,
            redirect: None,
            cgi_extension: None,
            cookie_required: false,
        },
        RouteConfig {
            path: "/admin".to_string(),
            methods: vec![Method::GET],
            root: "./www".to_string(),
            index_file: Some("admin.html".to_string()),
            directory_listing: false,
            redirect: None,
            cgi_extension: None,
            cookie_required: true,
        },
        RouteConfig {
            path: "/cgi".to_string(),
            methods: vec![Method::GET, Method::POST],
            root: "./cgi-scripts".to_string(),
            index_file: None,
            directory_listing: false,
            redirect: None,
            cgi_extension: Some(".py".to_string()),
            cookie_required: false,
        },
        //route for post ./uploads
        RouteConfig {
            path: "/uploads".to_string(),
            methods: vec![Method::POST],
            root: "./www/uploads".to_string(),
            index_file: None,
            directory_listing: true,
            redirect: None,
            cgi_extension: None,
            cookie_required: false,
        },
        RouteConfig {
            path: "/uploads_entries".to_string(),
            methods: vec![Method::GET],
            root: "./www".to_string(),
            index_file: Some("uploads.html".to_string()),
            directory_listing: false,
            redirect: None,
            cgi_extension: None,
            cookie_required: false,
        },
        RouteConfig {
            path: "/uploads/list".to_string(),
            methods: vec![Method::GET],
            root: "./www/uploads".to_string(),
            index_file: None,
            directory_listing: true,
            redirect: None,
            cgi_extension: None,
            cookie_required: false,
        },
        RouteConfig {
            path: "/uploads/delete".to_string(),
            methods: vec![Method::DELETE],
            root: "./www/uploads".to_string(),
            index_file: None,
            directory_listing: false, 
            redirect: None,
            cgi_extension: None,
            cookie_required: false,
        },
    ];

    let server = ServerConfig {
        host: "127.0.0.1".parse().unwrap(),
        ports: vec![8080],
        server_name: "localhost".to_string(),
        error_pages: HashMap::new(),
        client_max_body_size: 10 * 1024 * 1024,
        routes,
    };

    AppConfig { servers: vec![server] }
}

fn main() {


    ERROR_TEMPLATE
            .set(std::fs::read_to_string("./www/error.html").unwrap())
            .unwrap();

    let config = match parse_config_file("./config/server.conf") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            eprintln!("Using temporary hardcoded test config");
            hardcoded_test_config()
        }
    };

    let mut listeners = Vec::new();
    let mut unique_ports = HashSet::new();
    // we use hashset because we need only one listener for each port even if two servers use it

    for server in &config.servers {
        for port in &server.ports {
            if unique_ports.insert(*port) {
                let addr = format!("127.0.0.1:{}", port);
                listeners.push(server::create_listener(&addr));
            }
        }
    }

    event_loop::run(listeners, &config);
}

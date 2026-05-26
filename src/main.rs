mod server;
mod event_loop;
mod http;
mod client;
mod router;
mod config;
mod config_parser;

use crate::config::AppConfig;

fn main() {
    // start the server
    let listener1 = server::create_listener("127.0.0.1:8080");
    let listener2 = server::create_listener("127.0.0.1:8081");
    
    // run the event loop
    let config = AppConfig { servers: vec![/* ... */] };
    event_loop::run(vec![listener1, listener2], &config);
}

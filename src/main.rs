mod server;
mod event_loop;
mod http;
use std::sync::OnceLock;

pub static ERROR_TEMPLATE: OnceLock<String> = OnceLock::new();

fn main() {

    //load error template once at startup, and use it for all error responses
    ERROR_TEMPLATE
        .set(std::fs::read_to_string("www/error.html").unwrap())
        .unwrap();

    // start the server
    let listener = server::create_listener("127.0.0.1:8080");
    
    // run the event loop
    event_loop::run(listener);
}

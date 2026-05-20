mod server;
mod event_loop;
mod http;

fn main() {
    // start the server
    let listener = server::create_listener("127.0.0.1:8080");
    
    // run the event loop
    event_loop::run(listener);
}

mod server;
mod event_loop;
mod http;

fn main() {
    // start the server
    let listener1 = server::create_listener("127.0.0.1:8080");
    let listener2 = server::create_listener("127.0.0.1:8081");
    
    // run the event loop
    event_loop::run(vec![listener1, listener2]);
}

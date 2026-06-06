use localhost::server;
use localhost::event_loop;
use localhost::config_parser::parse_config_file;
use std::collections::HashSet;
use localhost::ERROR_TEMPLATE;

fn main() {


    ERROR_TEMPLATE
            .set(std::fs::read_to_string("./www/error.html").unwrap())
            .unwrap();

     match parse_config_file("config/server.conf") {
        // parse_config_file returns an AppConfig struct

        Ok(config) => {
            // println!("{:#?}", config);
            let mut listeners = Vec::new();
            let mut unique_ports = HashSet::new();
            // we use hashnet bc we need only one listener for each port even if two servers use it
            
            for server in &config.servers {
                for port in &server.ports {
                    if unique_ports.insert(*port) {
                        let addr = format!("{}:{}", server.host, port);
                        listeners.push(server::create_listener(&addr));
                    }
                }
            }
            
            event_loop::run(listeners, &config);
        }
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    }
}

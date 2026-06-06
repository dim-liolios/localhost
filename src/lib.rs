pub mod server;
pub mod event_loop;
pub mod http;
pub mod client;
pub mod router;
pub mod config;
pub mod config_parser;

use std::sync::OnceLock;
pub static ERROR_TEMPLATE: OnceLock<String> = OnceLock::new();
use crate::config::{AppConfig, ServerConfig, RouteConfig};

pub fn resolve_route<'a>(port: u16, server_name: &str, path: &str, config: &'a AppConfig) -> Option<(&'a ServerConfig, &'a RouteConfig)> {
    let server = find_server(port, server_name, &config.servers)?;
    let route = find_route(path, &server.routes)?;

    Some((server, route))
}

fn find_route<'a>(path: &str, routes: &'a [RouteConfig]) -> Option<&'a RouteConfig> {

    // 1. exact match:
    if let Some(route) = routes.iter().find(|route| route.path == path) {
        return Some(route);
    }

    // 2. longest prefix match: 
    routes.iter()
        .filter(|route| {
            path.starts_with(&route.path) && path.as_bytes().get(route.path.len()) == Some(&b'/')
        })
        .max_by_key(|route| route.path.len())
}

fn find_server<'a>(port: u16, server_name: &str, servers: &'a [ServerConfig]) -> Option<&'a ServerConfig> {
    let normalized_server_name = normalize_host(server_name);

    // 1. exact match:
    if let Some (server) = servers.iter().find(|server| {
        server.ports.contains(&port) && server.server_name == normalized_server_name
    }) {
        return Some(server);
    }

    // fallback: find any server listening on the port (for requests without Host header or unmatched server_name)
    servers.iter().find(|server| server.ports.contains(&port))

}

// ====================================================================================================================
// HELPER FUNCTIONS:

fn normalize_host(host: &str) -> String {
    host
        .trim()
        .to_lowercase()
        .split(':')
        .next()
        .unwrap_or("")
        .trim_end_matches('.')
        .to_string()
}

/* ====================================================================================================================
NOTES:

- get(route.path.len()) == Some(&b'/'):
    if the path is "/api" this checks the last character right after the prefix for the current route:
        if it's '/' -> valid sub-route (/api/users)
        if it's anything else -> invalid match (/apix)
    so first case passes the filtering and second one fails

- normalize_host():
    "localhost:8080" → "localhost"
    "Example.COM:8080." → "example.com"
    This ensures Host header matching works even with port numbers and case variations

*/
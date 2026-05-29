use crate::config::{AppConfig, ServerConfig, RouteConfig, Method};
use crate::config_parser::{tokenize, ParseError};
use std::collections::HashMap;
use std::net::IpAddr;

pub fn parse_config_file(path: &str) -> Result<AppConfig, ParseError> {
    let input = std::fs::read_to_string(path)
        .map_err(|e| ParseError::new(0, format!("Failed to read config file: {}", e)))?;
        // read server.conf file and return error (in ParseError format) if it fails. if Ok return it as a string in "input"

    let tokens = tokenize(&input)?;
    let mut parser = ConfigParser::new(tokens);
    let servers = parser.parse_servers()?;

    Ok(AppConfig { servers })
}

// ====================================================================================================================
// PARSER

pub struct ConfigParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl ConfigParser {
    fn new(tokens: Vec<Token>) -> Self {
        ConfigParser { tokens, pos: 0 }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect_word(&mut self, expected: &str) -> Result<(), ParseError> {
        match self.current() {
            Some(Token::Word(w)) if w == expected => {
                self.advance();
                Ok(())
            }
            Some(Token::Word(w)) => Err(ParseError::new(0, format!("Expected '{}', got '{}'", expected, w))),
            other => Err(ParseError::new(0, format!("Expected '{}', got {:?}", expected, other))),
        }
    }

    fn expect_open_brace(&mut self) -> Result<(), ParseError> {
        match self.current() {
            Some(Token::OpenBrace) => {
                self.advance();
                Ok(())
            }
            other => Err(ParseError::new(0, format!("Expected '{{', got {:?}", other))),
        }
    }

    fn expect_close_brace(&mut self) -> Result<(), ParseError> {
        match self.current() {
            Some(Token::CloseBrace) => {
                self.advance();
                Ok(())
            }
            other => Err(ParseError::new(0, format!("Expected '}}', got {:?}", other))),
        }
    }

    fn next_word(&mut self) -> Result<String, ParseError> {
        match self.current() {
            Some(Token::Word(w)) => {
                let result = w.clone();
                self.advance();
                Ok(result)
            }
            other => Err(ParseError::new(0, format!("Expected word, got {:?}", other))),
        }
    }

    fn parse_servers(&mut self) -> Result<Vec<ServerConfig>, ParseError> {
        let mut servers = Vec::new();

        while self.current().is_some() {
            self.expect_word("server")?;
            self.expect_open_brace()?;
            let server = self.parse_server()?;
            self.expect_close_brace()?;
            servers.push(server);
        }

        Ok(servers)
    }

    fn parse_server(&mut self) -> Result<ServerConfig, ParseError> {
        let mut host: Option<IpAddr> = None;
        let mut ports: Vec<u16> = Vec::new();
        let mut server_name = String::new();
        let mut client_max_body_size = 10 * 1024 * 1024; // 10MB default
        let mut error_pages: HashMap<u16, String> = HashMap::new();
        let mut routes: Vec<RouteConfig> = Vec::new();
        let mut default_route: Option<RouteConfig> = None;

        while self.current() != Some(&Token::CloseBrace) {
            match self.current() {
                Some(Token::Word(w)) => {
                    match w.as_str() {
                        "host" => {
                            self.advance();
                            let host_str = self.next_word()?;
                            host = Some(host_str.parse().map_err(|_| {
                                ParseError::new(0, format!("Invalid IP address: {}", host_str))
                            })?);
                        }
                        "ports" => {
                            self.advance();
                            loop {
                                match self.current() {
                                    Some(Token::Word(p)) => {
                                        let port: u16 = p.parse().map_err(|_| {
                                            ParseError::new(0, format!("Invalid port: {}", p))
                                        })?;
                                        ports.push(port);
                                        self.advance();
                                    }
                                    Some(Token::OpenBrace) | Some(Token::CloseBrace) | None => break,
                                    _ => break,
                                }
                            }
                        }
                        "server_name" => {
                            self.advance();
                            server_name = self.next_word()?;
                        }
                        "client_max_body_size" => {
                            self.advance();
                            let size_str = self.next_word()?;
                            client_max_body_size = size_str.parse().map_err(|_| {
                                ParseError::new(0, format!("Invalid size: {}", size_str))
                            })?;
                        }
                        "error_page" => {
                            self.advance();
                            let code_str = self.next_word()?;
                            let code: u16 = code_str.parse().map_err(|_| {
                                ParseError::new(0, format!("Invalid error code: {}", code_str))
                            })?;
                            let path = self.next_word()?;
                            error_pages.insert(code, path);
                        }
                        "route" => {
                            self.advance();
                            let route = self.parse_route()?;
                            routes.push(route);
                        }
                        _ => {
                            return Err(ParseError::new(0, format!("Unknown directive: {}", w)));
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        // Validate required fields
        if host.is_none() {
            return Err(ParseError::new(0, "Missing required 'host' directive"));
        }
        if ports.is_empty() {
            return Err(ParseError::new(0, "Missing required 'ports' directive"));
        }
        if server_name.is_empty() {
            return Err(ParseError::new(0, "Missing required 'server_name' directive"));
        }

        Ok(ServerConfig {
            host: host.unwrap(),
            ports,
            server_name,
            error_pages,
            client_max_body_size,
            routes,
        })
    }

    //     pub struct ServerConfig {
    //     pub host: IpAddr,
    //     pub ports: Vec<u16>,
    //     pub server_name: String,
    //     pub error_pages: HashMap<u16, String>,
    //     pub client_max_body_size: usize,
    //     pub routes: Vec<RouteConfig>,
    //     }

    fn parse_route(&mut self) -> Result<RouteConfig, ParseError> {
        let path = self.next_word()?;
        self.expect_open_brace()?;
        let mut route = self.parse_route_body()?;
        route.path = path;
        self.expect_close_brace()?;
        Ok(route)
    }

    fn parse_route_body(&mut self) -> Result<RouteConfig, ParseError> {
        let mut methods: Vec<Method> = Vec::new();
        let mut root = String::new();
        let mut index_file: Option<String> = None;
        let mut directory_listing = false;
        let mut redirect: Option<(u16, String)> = None;
        let mut cgi_extension: Option<String> = None;

        while self.current() != Some(&Token::CloseBrace) {
            match self.current() {
                Some(Token::Word(w)) => {
                    match w.as_str() {
                        "methods" => {
                            self.advance();
                            loop {
                                match self.current() {
                                    Some(Token::Word(m)) => {
                                        methods.push(match m.as_str() {
                                            "GET" => Method::GET,
                                            "POST" => Method::POST,
                                            "DELETE" => Method::DELETE,
                                            _ => return Err(ParseError::new(0, format!("Unknown method: {}", m))),
                                        });
                                        self.advance();
                                    }
                                    Some(Token::OpenBrace) | Some(Token::CloseBrace) | None => break,
                                    _ => break,
                                }
                            }
                        }
                        "root" => {
                            self.advance();
                            root = self.next_word()?;
                        }
                        "index" => {
                            self.advance();
                            index_file = Some(self.next_word()?);
                        }
                        "directory_listing" => {
                            self.advance();
                            let val = self.next_word()?;
                            directory_listing = val == "on";
                        }
                        "redirect" => {
                            self.advance();
                            let code_str = self.next_word()?;
                            let code: u16 = code_str.parse().map_err(|_| {
                                ParseError::new(0, format!("Invalid redirect code: {}", code_str))
                            })?;
                            let target = self.next_word()?;
                            redirect = Some((code, target));
                        }
                        "cgi" => {
                            self.advance();
                            cgi_extension = Some(self.next_word()?);
                        }
                        _ => {
                            return Err(ParseError::new(0, format!("Unknown route directive: {}", w)));
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        Ok(RouteConfig {
            path: String::new(), // Set by parse_route
            methods,
            root,
            index_file,
            directory_listing,
            redirect,
            cgi_extension,
        })
    }
}

// ====================================================================================================================
// PUBLIC API


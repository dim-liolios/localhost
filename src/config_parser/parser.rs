use crate::config::{AppConfig, ServerConfig, RouteConfig, Method};
use crate::config_parser::{tokenize, ParseError, Token};
use std::collections::HashMap;
use std::net::IpAddr;

pub fn parse_config_file(path: &str) -> Result<AppConfig, ParseError> {
    let input = std::fs::read_to_string(path)
        .map_err(|e| ParseError::new(0, format!("Failed to read config file: {}", e)))?;
        // read server.conf file and return error (in ParseError format) if it fails. if Ok return it as a string in "input"

    let tokens = tokenize(&input)?;

    // DEBUGGING
    // for (i, (token, line)) in tokens.iter().take(50).enumerate() {
    //     eprintln!("  [{}] Line {}: {:?}", i, line, token);
    // }

    let mut parser = ConfigParser::new(tokens);
    let servers = parser.parse_servers()?;

    Ok(AppConfig { servers })
}

pub struct ConfigParser {
    tokens: Vec<Token>,
    lines: Vec<usize>, // line number for each token
    pos: usize,
}

// ConfigParser STRUCT METHODS ========================================================================================
impl ConfigParser {

    // TOKEN METHODS ----------------------------------------------------------------------------------------
    // create a new ConfigParser with the given tokens and set the position to 0
    fn new(token_pairs: Vec<(Token, usize)>) -> Self {
        let (tokens, lines) = token_pairs.into_iter().unzip(); // unzip token_pairs into separate tokens and lines vectors
        ConfigParser { tokens, lines, pos: 0 }
    }

    // get the current token
    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
        // get() is a vec method that returns an Option: Some(&Token) if the index is valid, None if it's out of bounds
        // if we had used self.tokens[self.pos] instead, it would panic if pos were out of bounds
    }

    // get the current line number
    fn current_line(&self) -> usize {
        self.lines.get(self.pos).copied().unwrap_or(0)
    }

    // advance parser (ConfigParser) position to the next token
    fn advance_parser(&mut self) {
        self.pos += 1;
    }

    // EXPECT + CONSUME for Tokens METHODS ------------------------------------------------------------------
    // all expect/consume methods include advance_parser() to move to the next token after successfully matching/consuming the current one.
    fn expect_word(&mut self, expected: &str) -> Result<(), ParseError> {
        match self.current() {
            Some(Token::Word(w)) if w == expected => {
                self.advance_parser();
                Ok(())
            }
            Some(Token::Word(w)) => Err(ParseError::new(self.current_line(), format!("Expected '{}', got '{}'", expected, w))),
            other => Err(ParseError::new(self.current_line(), format!("Expected '{}', got {:?}", expected, other))),
        }
    }

    fn expect_open_brace(&mut self) -> Result<(), ParseError> {
        match self.current() {
            Some(Token::OpenBrace) => {
                self.advance_parser();
                Ok(())
            }
            other => Err(ParseError::new(self.current_line(), format!("Expected '{{', got {:?}", other))),
        }
    }

    fn expect_close_brace(&mut self) -> Result<(), ParseError> {
        match self.current() {
            Some(Token::CloseBrace) => {
                self.advance_parser();
                Ok(())
            }
            other => Err(ParseError::new(self.current_line(), format!("Expected '}}', got {:?}", other))),
        }
    }
    
    fn consume_word(&mut self) -> Result<String, ParseError> {
        match self.current() {
            Some(Token::Word(w)) => {
                let result = w.clone();
                self.advance_parser();
                Ok(result)
            }
            other => Err(ParseError::new(self.current_line(), format!("Expected word, got {:?}", other))),
        }
        // 1. check if the current token is a Word
        // 2. clone the String (to return it) and advance() the parser
        // 3. return current word as String
    }
    
    // PARSE SERVER METHODS ---------------------------------------------------------------------------------
    fn parse_servers(&mut self) -> Result<Vec<ServerConfig>, ParseError> {
        let mut servers = Vec::new();

        while self.current().is_some() { // is_some is an Option method that returns true if the Option is Some, false if it's None
            self.expect_word("server")?; // we expect the first token to be "server" (if not, return an error and stop parsing)
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
        let mut error_pages: HashMap<u16, String> = HashMap::new();
        let mut client_max_body_size = 10 * 1024 * 1024; // 10MB default
        let mut routes: Vec<RouteConfig> = Vec::new();

        while self.current() != Some(&Token::CloseBrace) {
            match self.current() {
                Some(Token::Word(w)) => {
                    match w.as_str() { // as_str() is a String method that converts the String to a &str
                        "host" => {    // the only reason we do this is to be able to match w (String) tokens with literals like "host", "ports", etc
                            self.advance_parser();
                            let host_str = self.consume_word()?; // host_str is String
                            host = Some(host_str.parse().map_err(|_| { 
                                // parse() is a String method that converts the String to the specified type (in this case, IpAddr)
                                ParseError::new(self.current_line(), format!("Invalid IP address: {}", host_str))
                            })?);
                        }
                        "ports" => {
                            self.advance_parser();
                            loop {
                                match self.current() {
                                    Some(Token::Word(p)) => {
                                        match p.parse::<u16>() {
                                            Ok(port) => {
                                                if ports.contains(&port) {
                                                    return Err(ParseError::new(self.current_line(), format!("Duplicate port in server block: {}", port)));
                                                }
                                                ports.push(port);
                                                self.advance_parser();
                                            }
                                            Err(_) => break,
                                        }
                                    }
                                    _ => break,
                                }
                            }
                        }
                        "server_name" => {
                            self.advance_parser();
                            server_name = self.consume_word()?;
                        }
                        "client_max_body_size" => {
                            self.advance_parser();
                            let size_str = self.consume_word()?;
                            client_max_body_size = size_str.parse().map_err(|_| {
                                ParseError::new(self.current_line(), format!("Invalid size: {}", size_str))
                            })?;
                        }
                        "error_page" => {
                            self.advance_parser();
                            let code_str = self.consume_word()?;
                            let code: u16 = code_str.parse().map_err(|_| {
                                ParseError::new(self.current_line(), format!("Invalid error code: {}", code_str))
                            })?;
                            let path = self.consume_word()?;
                            error_pages.insert(code, path);
                        }
                        "route" => {
                            self.advance_parser();
                            let route = self.parse_route()?;
                            routes.push(route);
                        }
                        _ => {
                            return Err(ParseError::new(self.current_line(), format!("Unknown directive: {}", w)));
                        }
                    }
                }
                _ => {
                    return Err(ParseError::new(self.current_line(), match self.current() {
                        None => "Unexpected end of file, missing '}'".to_string(),
                        other => format!("Unexpected token: {:?}", other),
                        }));
                }
            }
        }

        // validate only required fields
        if host.is_none() {
            return Err(ParseError::new(self.current_line(), "Missing required 'host' directive"));
        }
        if ports.is_empty() {
            return Err(ParseError::new(self.current_line(), "Missing required 'ports' directive"));
        }
        if server_name.is_empty() {
            return Err(ParseError::new(self.current_line(), "Missing required 'server_name' directive"));
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

    // PARSE ROUTE METHODS ----------------------------------------------------------------------------------
    fn parse_route(&mut self) -> Result<RouteConfig, ParseError> {
        let path = self.consume_word()?; // for example: "/images"
        self.expect_open_brace()?;
        let mut route = self.parse_route_body()?; // we create a RouteConfig struct here, in next line we add its path field
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
        let mut cookie_required = false;

        while self.current() != Some(&Token::CloseBrace) {
            match self.current() {
                Some(Token::Word(w)) => {
                    match w.as_str() {
                        "methods" => {
                            self.advance_parser();
                            loop {
                                match self.current() {
                                    Some(Token::Word(m)) => {
                                        methods.push(match m.as_str() {
                                            "GET" => Method::GET,
                                            "POST" => Method::POST,
                                            "DELETE" => Method::DELETE,
                                            _ => break,
                                        });
                                        self.advance_parser();
                                    }
                                    _ => break,
                                }
                            }
                        }
                        "root" => { // root is required for RouteConfig struct
                            self.advance_parser();
                            root = self.consume_word()?;
                        }
                        "index" => { // index is not required for RouteConfig struct (so it can be None)
                            self.advance_parser();
                            index_file = Some(self.consume_word()?);
                        }
                        "directory_listing" => {
                            self.advance_parser();
                            let val = self.consume_word()?;
                            directory_listing = val == "on"; // if val = "on" then directory_listing = true, else false
                        }
                        "redirect" => {
                            self.advance_parser();
                            let status_code_str = self.consume_word()?;
                            let status_code: u16 = status_code_str.parse().map_err(|_| {
                                ParseError::new(self.current_line(), format!("Invalid redirect code: {}", status_code_str))
                            })?;
                            let target_url = self.consume_word()?;
                            redirect = Some((status_code, target_url));
                        }
                        "cgi" => {
                            self.advance_parser();
                            cgi_extension = Some(self.consume_word()?);
                        }
                        "cookie_required" => {
                            self.advance_parser();
                            let val = self.consume_word()?;
                            cookie_required = val == "on";
                        }
                        _ => {
                            return Err(ParseError::new(self.current_line(), format!("Unknown route directive: {}", w)));
                        }
                    }
                }
                _ => {
                    return Err(ParseError::new(self.current_line(), match self.current() {
                        None => "Unexpected end of file, missing '}'".to_string(),
                        other => format!("Unexpected token: {:?}", other),
                        }));
                }
            }
        }

        if redirect.is_none() && root.is_empty() {
            return Err(ParseError::new(self.current_line(), "Route must have either 'root' or 'redirect' directive"));
        }

        Ok(RouteConfig {
            path: String::new(), // set inside  parse_route(): route.path = path;
            methods,
            root,
            index_file,
            directory_listing,
            redirect,
            cgi_extension,
            cookie_required,
        })
    }
}

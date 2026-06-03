use localhost::config_parser::{parse_config_file, tokenize, Token};

// server.conf parsing tests
#[test]
fn parses_full_config() {
    let config = parse_config_file("config/server.conf").unwrap();

    assert_eq!(config.servers.len(), 2);

    // server 1
    let s1 = &config.servers[0];
    assert_eq!(s1.host.to_string(), "127.0.0.1");
    assert_eq!(s1.ports, vec![8080, 8081]);
    assert_eq!(s1.server_name, "localhost");

    // routes exist
    assert!(s1.routes.iter().any(|r| r.path == "/"));
    assert!(s1.routes.iter().any(|r| r.path == "/uploads"));
}

// test the tokenizer separately
#[test]
#[test]
fn tokenizer_basic() {
    let input = "server { host 127.0.0.1 }";
    let tokens = tokenize(input).unwrap();

    assert!(matches!(tokens[0], (Token::Word(ref w), _) if w == "server"));
    assert!(matches!(tokens[1], (Token::OpenBrace, _)));
}

#[test]
fn route_parsing() {
    let config = parse_config_file("config/server.conf").unwrap();

    let route = config.servers[0]
        .routes
        .iter()
        .find(|r| r.path == "/redirect")
        .unwrap();

    assert!(route.redirect.is_some());
}
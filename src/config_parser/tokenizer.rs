use crate::config_parser::error::ParseError;

// each token is either a word, an open brace, or a close brace

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    OpenBrace,
    CloseBrace,
}

fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::new();
    let mut current_word = String::new();
    let mut line = 1;

    for ch in input.chars() {
        match ch {
            '{' => {
                if !current_word.is_empty() {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                }
                tokens.push(Token::OpenBrace);
            }
            '}' => {
                if !current_word.is_empty() {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                }
                tokens.push(Token::CloseBrace);
            }
            ' ' | '\t' => {
                if !current_word.is_empty() {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                }
            }
            '\n' => {
                if !current_word.is_empty() {
                    tokens.push(Token::Word(current_word.clone()));
                    current_word.clear();
                }
                line += 1;
            }
            '#' => {
                // Comment: skip until end of line
                for remaining_ch in input.chars() {
                    if remaining_ch == '\n' {
                        line += 1;
                        break;
                    }
                }
            }
            _ => {
                current_word.push(ch);
            }
        }
    }

    if !current_word.is_empty() {
        tokens.push(Token::Word(current_word));
    }

    Ok(tokens)
}
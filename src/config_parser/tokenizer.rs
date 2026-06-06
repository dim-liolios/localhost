use crate::config_parser::error::ParseError;

// each token is either a word, an open brace, or a close brace

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    OpenBrace,
    CloseBrace,
}

pub fn tokenize(input: &str) -> Result<Vec<(Token, usize)>, ParseError> {
    let mut tokens = Vec::<(Token, usize)>::new();
    let mut current_word = String::new();
    let mut line = 1;

    for character in input.chars() {
        match character {
            '{' => {
                if !current_word.is_empty() {
                    tokens.push((Token::Word(current_word.clone()), line));
                    current_word.clear();
                }
                tokens.push((Token::OpenBrace, line));
            }
            '}' => {
                if !current_word.is_empty() {
                    tokens.push((Token::Word(current_word.clone()), line));
                    current_word.clear();
                }
                tokens.push((Token::CloseBrace, line));
            }
            ' ' | '\t' => { // whitespace: end of a word
                if !current_word.is_empty() {
                    tokens.push((Token::Word(current_word.clone()), line));
                    current_word.clear();
                }
            }
            '\n' | '\r'=> {
                if !current_word.is_empty() {
                    tokens.push((Token::Word(current_word.clone()), line));
                    current_word.clear();
                }
                if character == '\n' {
                    line += 1;
                }
            }
            _ => {
                current_word.push(character);
            }
        }
    }

    if !current_word.is_empty() {
        tokens.push((Token::Word(current_word), line));
    }

    Ok(tokens)
}

/* ====================================================================================================================
NOTES:

- _ => {
    current_word.push(character);
        }
    every character we find is added in current_word until we hit a whitespace, a brace, or a comment
    then we push the current word as a token (if it's not empty) and clear it for the next word

*/
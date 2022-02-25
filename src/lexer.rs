use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Number(isize), // 整数
    Plus,          // +
    Minus,         // -
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

pub struct LexerError {
    pub msg: String,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a String) -> Lexer<'a> {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut result = vec![];

        loop {
            let token = self.next_token();

            match token {
                Ok(Some(token)) => {
                    result.push(token);
                }
                Err(e) => {
                    return Err(e);
                }
                _ => {
                    break;
                }
            }
        }

        Ok(result)
    }

    fn next_token(&mut self) -> Result<Option<Token>, LexerError> {
        while let Some(c) = self.chars.clone().peek() {
            match c {
                c if c.is_whitespace() => {
                    self.chars.next();
                }
                '+' => {
                    return Ok(self.next_return_token(Token::Plus));
                }
                '-' => {
                    return Ok(self.next_return_token(Token::Minus));
                }
                c if c.is_numeric() => match take_first_num(&mut self.chars) {
                    Some(num) => match num.parse::<isize>() {
                        Ok(num) => {
                            return Ok(Some(Token::Number(num)));
                        }
                        Err(e) => {
                            return Err(LexerError::new(e.to_string()));
                        }
                    },
                    None => {
                        return Err(LexerError::new(format!(
                            "文字列の読み込みに失敗しました: {}",
                            c
                        )));
                    }
                },
                c => {
                    return Err(LexerError::new(format!("予期しない文字です: {}", c)));
                }
            }
        }

        Ok(None)
    }

    fn next_return_token(&mut self, token: Token) -> Option<Token> {
        self.chars.next();
        Some(token)
    }
}

impl LexerError {
    fn new(msg: String) -> LexerError {
        LexerError { msg }
    }
}

fn take_first_num<'a>(input: &mut Peekable<Chars<'a>>) -> Option<String> {
    let mut result = String::new();

    while let Some(c) = input.clone().peek() {
        match c {
            c if c.is_numeric() => {
                result.push(*c);
                input.next();
            }
            _ => {
                if result.len() == 0 {
                    return None;
                }

                return Some(result);
            }
        }
    }

    return Some(result);
}

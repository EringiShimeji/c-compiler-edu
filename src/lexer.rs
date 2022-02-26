use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Number(isize), // 整数
    Plus,          // +
    Minus,         // -
}

pub struct Lexer<'a> {
    chars: Peekable<Enumerate<Chars<'a>>>,
}

pub struct LexerError {
    msg: String,
    is_primitive_error: bool,
    at: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a String) -> Lexer<'a> {
        Lexer {
            chars: input.chars().enumerate().peekable(),
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
        while let Some((i, c)) = self.chars.clone().peek() {
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
                            return Err(LexerError::new(e.to_string(), true, 0));
                        }
                    },
                    None => {
                        return Err(LexerError::new(
                            format!("文字列の読み込みに失敗しました: {}", c),
                            false,
                            *i,
                        ));
                    }
                },
                c => {
                    return Err(LexerError::new(
                        format!("予期しない文字です: {}", c),
                        false,
                        *i,
                    ));
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
    fn new(msg: String, is_primitive_error: bool, at: usize) -> LexerError {
        LexerError {
            msg,
            is_primitive_error,
            at,
        }
    }

    pub fn get_msg(&self, input: &String) -> String {
        if self.is_primitive_error {
            return self.msg.clone();
        }

        let mut arrow = String::new();

        for _ in 0..self.at {
            arrow.push(' ');
        }

        arrow.push('^');

        let result = format!("{}\n{} {}", input, arrow, self.msg);

        result
    }
}

fn take_first_num<'a>(input: &mut Peekable<Enumerate<Chars<'a>>>) -> Option<String> {
    let mut result = String::new();

    while let Some((_, c)) = input.clone().peek() {
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

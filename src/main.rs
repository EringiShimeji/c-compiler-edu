mod lexer;

use lexer::Lexer;
use std::{env, process};

use crate::lexer::Token;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    let mut tokens = match Lexer::new(&args[1]).tokenize() {
        Ok(tokens) => tokens.into_iter().peekable(),
        Err(e) => {
            exit_with_error(e.msg);
            Vec::new().into_iter().peekable()
        }
    };

    if tokens.len() > 0 {
        match tokens.next() {
            Some(Token::Number(num)) => {
                println!("  mov rax, {}", num);
            }
            Some(Token::Plus) => {
                exit_with_error("予期しないトークン: +");
            }
            Some(Token::Minus) => {
                exit_with_error("予期しないトークン: -");
            }
            None => {
                exit_with_error("引数が不正です");
            }
        }
    }

    while let Some(token) = tokens.clone().peek() {
        match token {
            Token::Plus | Token::Minus => {
                tokens.next();

                if let Some(Token::Number(num)) = tokens.next() {
                    let op = match token {
                        Token::Plus => "add",
                        Token::Minus => "sub",
                        _ => {
                            panic!("予期しないトークンが読み込まれました")
                        }
                    };

                    println!("  {} rax, {}", op, num);
                } else {
                    exit_with_error(format!("数値が必要です"));
                }
            }
            Token::Number(num) => {
                exit_with_error(format!("予期しないトークン: {}", num));
            }
        }
    }

    println!("  ret");
}

fn exit_with_error(msg: impl std::fmt::Display) {
    eprintln!("{}", msg);
    process::exit(1);
}

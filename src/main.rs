mod codegen;
mod lexer;
mod parser;

use lexer::Lexer;
use std::{env, fmt, process};

use crate::{codegen::gen, parser::Parser};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }

    // 字句解析器を初期化
    let mut lexer = Lexer::new(&args[1]);

    // トークナイズしつつエラーがあればプログラムを止める
    if let Err(msg) = lexer.tokenize() {
        error(&mut lexer, msg);
    }

    // パーサーを初期化
    let mut parser = Parser::new(lexer);
    let node = match parser.expr() {
        Ok(node) => node,
        Err(msg) => {
            error(&mut parser.get_lexer(), msg);
            return;
        }
    };

    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    gen(node);

    // スタックトップに式全体の値が残っているはずなので、RAXにロードして関数からの返り値とする
    println!("  pop rax");
    println!("  ret");
}

fn error<'a>(lexer: &mut Lexer<'a>, msg: impl fmt::Display) {
    let msg = lexer.error_at(msg);
    eprintln!("{}", msg);
    process::exit(1);
}

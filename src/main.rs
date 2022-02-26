use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }

    let num = match args[1].parse::<isize>() {
        Ok(n) => n,
        Err(_) => 0,
    };

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    println!("  mov rax, {}", num);
    println!("  ret");
}

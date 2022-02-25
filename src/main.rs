use std::{env, iter::Peekable, process, str::Chars};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    let mut input = args[1].chars().peekable();

    match take_first_num(&mut input) {
        Some(num) => {
            println!("  mov rax, {}", num);
        }
        None => {
            eprintln!("引数が不正です");
            process::exit(1);
        }
    }

    while let Some(op) = input.clone().peek() {
        match op {
            '+' | '-' => {
                input.next();

                match take_first_num(&mut input) {
                    Some(num) => {
                        let op_command = if *op == '+' { "add" } else { "sub" };

                        println!("  {} rax, {}", op_command, num)
                    }
                    None => {
                        eprintln!("項が必要です");
                        process::exit(1)
                    }
                };
            }
            c => {
                eprintln!("予期しない文字です: {}", c);
                process::exit(1);
            }
        }
    }

    println!("  ret");
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

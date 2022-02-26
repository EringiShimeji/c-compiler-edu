use std::{env, iter::Peekable, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }

    let mut input = args[1].chars().peekable();

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    let num = match take_num_str(&mut input) {
        Ok(s) => s,
        Err((s, _)) => s,
    };

    println!("  mov rax, {}", num);

    while let Some(c) = input.peek() {
        match c {
            '+' | '-' => {
                let op = if *c == '+' { "add" } else { "sub" };

                input.next();

                let num = match take_num_str(&mut input) {
                    Ok(s) => s,
                    Err((s, _)) => s,
                };

                println!("  {} rax, {}", op, num);
            }
            _ => {
                eprintln!("予期しない文字です: '{}'", c);
                process::exit(1);
            }
        }
    }

    println!("  ret");
}

/// 文字列の先頭から始まる数値を取得し、何文字目まで読み込んだか返す
/// 0から始まる数字の羅列や、数字以外が含まれる場合は、途中まで読み込んだ数値と不正な文字をタプルとしてErrで返す
fn take_num_str<'a>(input: &mut Peekable<std::str::Chars<'a>>) -> Result<String, (String, char)> {
    let mut result = String::new();

    while let Some(c) = input.peek() {
        match c {
            c if c.is_numeric() => {
                if result.len() == 0 && *c == '0' {
                    input.next();

                    if let Some(next_char) = input.peek() {
                        return Err(("0".to_string(), *next_char));
                    } else {
                        return Ok("0".to_string());
                    }
                }

                result.push(*c);
                input.next();
            }
            c => {
                if result.len() == 0 {
                    return Err((result, *c));
                }

                break;
            }
        }
    }

    Ok(result)
}

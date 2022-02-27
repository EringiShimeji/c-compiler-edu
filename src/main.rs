use std::{env, fmt, iter::Peekable, process, str::Chars, vec::IntoIter};

#[derive(Clone, PartialEq, Eq)]
enum Reserved {
    Plus,
    Minus,
}

impl fmt::Display for Reserved {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Reserved::Plus => "+",
            Reserved::Minus => "-",
        };

        write!(f, "{}", s)
    }
}

#[derive(Clone)]
enum TokenKind {
    Reserved(Reserved), // 記号
    Num(isize),         // 整数とその値
    EOF,                // 入力の終わりを表すトークン
}

#[derive(Clone)]
struct Token<'a> {
    kind: TokenKind,            // トークンの型
    chars: Peekable<Chars<'a>>, // そのトークン以降の文字列
}

impl<'a> Token<'a> {
    fn new(kind: TokenKind, chars: Peekable<Chars<'a>>) -> Token<'a> {
        Token { kind, chars }
    }
}

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    tokens: Peekable<IntoIter<Token<'a>>>,
}

impl<'a> Lexer<'a> {
    fn new(s: &'a String) -> Lexer<'a> {
        Lexer {
            chars: s.chars().peekable(),
            tokens: vec![].into_iter().peekable(),
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token<'a>>, String> {
        let mut result: Vec<Token<'a>> = Vec::new();

        while let Some(c) = self.chars.clone().peek() {
            match c {
                c if c.is_whitespace() => {
                    self.chars.next();
                }
                '+' | '-' => {
                    let op = if *c == '+' {
                        TokenKind::Reserved(Reserved::Plus)
                    } else {
                        TokenKind::Reserved(Reserved::Minus)
                    };
                    let token = Token::new(op, self.chars.clone());

                    result.push(token);
                    self.chars.next();
                }
                c if c.is_numeric() => {
                    let num = match take_num_str(&mut self.chars) {
                        Ok(s) => s,
                        Err((s, _)) => s,
                    };

                    if let Ok(num) = num.parse::<isize>() {
                        let token = Token::new(TokenKind::Num(num), self.chars.clone());

                        result.push(token);
                    } else {
                        return Err("数ではありません".to_string());
                    }
                }
                _ => {
                    return Err("トークナイズできません".to_string());
                }
            }
        }

        result.push(Token::new(TokenKind::EOF, self.chars.clone()));

        // トークンを保存
        self.tokens = result.clone().into_iter().peekable();

        Ok(result)
    }

    fn at_eof(&mut self) -> bool {
        if let Some(Token {
            kind: TokenKind::EOF,
            ..
        }) = self.tokens.peek()
        {
            true
        } else {
            false
        }
    }

    /// 次のトークンが期待している記号の時は、トークンを1つ読み進めて真を返す
    /// それ以外の場合は偽を返す
    fn consume(&mut self, expect: Reserved) -> bool {
        if let Some(Token {
            kind: TokenKind::Reserved(op),
            ..
        }) = self.tokens.peek()
        {
            if *op == expect {
                self.tokens.next();

                return true;
            }
        }

        false
    }

    /// 次のトークンが期待している記号の時は、トークンを1つ読み進める
    /// それ以外の場合はエラーを報告する
    fn expect(&mut self, expect: Reserved) -> Result<(), String> {
        if let Some(Token {
            kind: TokenKind::Reserved(op),
            ..
        }) = self.tokens.peek()
        {
            if *op == expect {
                self.tokens.next();

                return Ok(());
            }
        }

        Err(format!("{}ではありません", expect))
    }

    /// 次のトークンが数値の場合、トークンを1つ読み進めてその数値を返す。
    /// それ以外の場合にはエラーを報告する。
    fn expect_number(&mut self) -> Result<isize, String> {
        if let Some(Token {
            kind: TokenKind::Num(num),
            ..
        }) = self.tokens.peek()
        {
            let num = *num;

            self.tokens.next();

            return Ok(num);
        }

        Err("数ではありません".to_string())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        process::exit(1);
    }

    // 解析器を初期化
    let mut lexer = Lexer::new(&args[1]);

    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // トークナイズしつつエラーがあればプログラムを止める
    if let Err(msg) = lexer.tokenize() {
        error(msg);
    }

    // 式の最初は数でなければならないので、それをチェックして最初のmov命令を出力
    match lexer.expect_number() {
        Ok(num) => {
            println!("  mov rax, {}", num);
        }
        Err(msg) => {
            error(msg);
        }
    }

    // `+ <数>`あるいは`- <数>`というトークンの並びを消費しつつアセンブリを出力
    while !lexer.at_eof() {
        if lexer.consume(Reserved::Plus) {
            match lexer.expect_number() {
                Ok(num) => {
                    println!("  add rax, {}", num);
                }
                Err(msg) => {
                    error(msg);
                }
            }

            continue;
        }

        if let Err(msg) = lexer.expect(Reserved::Minus) {
            error(msg);
        };

        match lexer.expect_number() {
            Ok(num) => {
                println!("  sub rax, {}", num);
            }
            Err(msg) => {
                error(msg);
            }
        }
    }

    println!("  ret");
}

/// 数値であるような文字列全体もしくは先頭から続く部分列を取り出す
/// 0から始まる数字の羅列や、数字以外が含まれる場合は、途中まで読み込んだ数値と不正な文字をタプルとしてErrで返す
/// 備考: C言語のstrtolの仕様を参考にした
fn take_num_str<'a>(input: &mut Peekable<std::str::Chars<'a>>) -> Result<String, (String, char)> {
    let mut result = String::new();

    while let Some(c) = input.peek() {
        match c {
            // 先頭の空白は無視する
            c if result.len() == 0 && c.is_whitespace() => {
                input.next();
            }

            // 符号付き整数の可能性がある
            '+' | '-' => {
                // 符号の位置が先頭なら、文字列全体もしくは先頭から続く部分列が整数である可能性がある
                if result.len() == 0 {
                    let op = *c;
                    let mut cloned = input.clone();

                    cloned.next();

                    if let Some(c) = cloned.next() {
                        match c {
                            // 符号の後に数字が続けば、符号付き整数であると評価する
                            c if c.is_numeric() => {
                                // `-0`は`0`とみなすため、先頭の符号は無視する
                                if op == '-' && c != '0' {
                                    result.push(op);
                                }

                                input.next();
                            }
                            _ => {
                                // 符号付き整数ではないなら、その符号の位置でエラーを返す
                                return Err((result, op));
                            }
                        }
                    } else {
                        return Err((result, op));
                    }
                } else {
                    return Err((result, *c));
                }
            }

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
                return Err((result, *c));
            }
        }
    }

    Ok(result)
}

fn error(msg: String) {
    eprintln!("{}", msg);
    process::exit(1);
}

#[cfg(test)]
mod test {
    use super::{take_num_str, Lexer};

    #[test]
    fn take_num_str_test() {
        let mut input = "123".chars().peekable();
        assert_eq!("123".to_string(), take_num_str(&mut input).unwrap());
        let mut input = "+456".chars().peekable();
        assert_eq!("456".to_string(), take_num_str(&mut input).unwrap());
        let mut input = "-789".chars().peekable();
        assert_eq!("-789".to_string(), take_num_str(&mut input).unwrap());
        let mut input = "0".chars().peekable();
        assert_eq!("0".to_string(), take_num_str(&mut input).unwrap());
        let mut input = "-0".chars().peekable();
        assert_eq!("0".to_string(), take_num_str(&mut input).unwrap());
        let mut input = "123a".chars().peekable();
        assert_eq!(
            ("123".to_string(), 'a'),
            take_num_str(&mut input).unwrap_err()
        );
        let mut input = "5+20".chars().peekable();
        assert_eq!(
            ("5".to_string(), '+'),
            take_num_str(&mut input).unwrap_err()
        );
    }
}

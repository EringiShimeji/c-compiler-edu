use std::{env, fmt, iter::Peekable, process, str::Chars, vec::IntoIter};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Reserved {
    LeftParen,
    RightParen,
    Plus,
    Minus,
    Asterisk,
    Slash,
}

impl fmt::Display for Reserved {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Reserved::LeftParen => "(",
            Reserved::RightParen => ")",
            Reserved::Plus => "+",
            Reserved::Minus => "-",
            Reserved::Asterisk => "*",
            Reserved::Slash => "/",
        };

        write!(f, "{}", s)
    }
}

struct ReservedError(char);

impl TryFrom<&char> for Reserved {
    type Error = ReservedError;

    fn try_from(item: &char) -> Result<Self, Self::Error> {
        match item {
            '(' => Ok(Reserved::LeftParen),
            ')' => Ok(Reserved::RightParen),
            '+' => Ok(Reserved::Plus),
            '-' => Ok(Reserved::Minus),
            '*' => Ok(Reserved::Asterisk),
            '/' => Ok(Reserved::Slash),
            _ => Err(ReservedError(*item)),
        }
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
    input: &'a String, // 入力プログラム
    chars: Peekable<Chars<'a>>,
    tokens: Peekable<IntoIter<Token<'a>>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a String) -> Lexer<'a> {
        Lexer {
            input,
            chars: input.chars().peekable(),
            tokens: vec![].into_iter().peekable(),
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token<'a>>, impl fmt::Display> {
        let mut result: Vec<Token<'a>> = Vec::new();

        while let Some(c) = self.chars.clone().peek() {
            match c {
                c if c.is_whitespace() => {
                    self.chars.next();
                }
                '(' | ')' | '+' | '-' | '*' | '/' => {
                    let reserved = Reserved::try_from(c);
                    match reserved {
                        Ok(reserved) => {
                            let token =
                                Token::new(TokenKind::Reserved(reserved), self.chars.clone());

                            result.push(token);
                            self.chars.next();
                        }
                        Err(_) => {
                            return Err("予期しない文字です");
                        }
                    }
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
                        return Err("数ではありません");
                    }
                }
                _ => {
                    return Err("トークナイズできません");
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
            kind: TokenKind::Reserved(reserved),
            ..
        }) = self.tokens.peek()
        {
            if *reserved == expect {
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
            kind: TokenKind::Reserved(reserved),
            ..
        }) = self.tokens.peek()
        {
            if *reserved == expect {
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

    /// 発生したエラー箇所を報告する
    fn error_at(&mut self, msg: impl fmt::Display) -> String {
        // トークナイズ中かトークンの消費中かを判別する
        let is_tokenizing = self.chars.peek() != None;
        let input = self.input.clone();
        // tokensが空なら元のプログラムの最後の位置でエラーを報告する
        let pos = if is_tokenizing {
            input.len() - self.chars.clone().count()
        } else {
            if let Some(token) = self.tokens.peek() {
                input.len() - token.chars.clone().count()
            } else {
                input.len()
            }
        };
        let msg_with_arrow = format!("{}^ {}", " ".repeat(pos), msg);

        return format!("{}\n{}", input, msg_with_arrow);
    }
}

/// 抽象構文木のノードの種類
enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Num(isize),
}

/// 抽象構文木のノード
struct Node {
    kind: NodeKind,         // ノードの型
    lhs: Option<Box<Node>>, // 左辺
    rhs: Option<Box<Node>>, // 右辺
}

impl Node {
    fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Node {
        Node { kind, lhs, rhs }
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    fn new(lexer: Lexer<'a>) -> Parser<'a> {
        Parser { lexer }
    }

    fn expr(&mut self) -> Result<Node, String> {
        let mut node = match self.mul() {
            Ok(node) => node,
            Err(msg) => {
                return Err(msg);
            }
        };

        loop {
            if self.lexer.consume(Reserved::Plus) {
                match self.mul() {
                    Ok(mul) => {
                        node = Node::new(NodeKind::Add, Some(Box::new(node)), Some(Box::new(mul)))
                    }
                    Err(msg) => {
                        return Err(msg);
                    }
                }
            } else if self.lexer.consume(Reserved::Minus) {
                match self.mul() {
                    Ok(mul) => {
                        node = Node::new(NodeKind::Sub, Some(Box::new(node)), Some(Box::new(mul)))
                    }
                    Err(msg) => {
                        return Err(msg);
                    }
                }
            } else {
                return Ok(node);
            }
        }
    }

    fn mul(&mut self) -> Result<Node, String> {
        let mut node = match self.primary() {
            Ok(node) => node,
            Err(msg) => {
                return Err(msg);
            }
        };

        loop {
            if self.lexer.consume(Reserved::Asterisk) {
                match self.primary() {
                    Ok(primary) => {
                        node =
                            Node::new(NodeKind::Mul, Some(Box::new(node)), Some(Box::new(primary)));
                    }
                    Err(msg) => {
                        return Err(msg);
                    }
                }
            } else if self.lexer.consume(Reserved::Slash) {
                match self.primary() {
                    Ok(primary) => {
                        node =
                            Node::new(NodeKind::Div, Some(Box::new(node)), Some(Box::new(primary)));
                    }
                    Err(msg) => {
                        return Err(msg);
                    }
                }
            } else {
                return Ok(node);
            }
        }
    }

    fn primary(&mut self) -> Result<Node, String> {
        if self.lexer.consume(Reserved::LeftParen) {
            let node = match self.expr() {
                Ok(node) => node,
                Err(msg) => {
                    return Err(msg);
                }
            };

            if let Err(msg) = self.lexer.expect(Reserved::RightParen) {
                return Err(msg);
            }

            return Ok(node);
        }

        if let Ok(num) = self.lexer.expect_number() {
            let node = Node::new(NodeKind::Num(num), None, None);

            return Ok(node);
        }

        Err("".to_string())
    }
}

fn gen(node: Node) {
    if let NodeKind::Num(num) = node.kind {
        println!("  push {}", num);
        return;
    }

    if let Some(lhs) = node.lhs {
        gen(*lhs);
    };

    if let Some(rhs) = node.rhs {
        gen(*rhs);
    };

    println!("  pop rdi");
    println!("  pop rax");

    match node.kind {
        NodeKind::Add => {
            println!("  add rax, rdi");
        }
        NodeKind::Sub => {
            println!("  sub rax, rdi");
        }
        NodeKind::Mul => {
            println!("  imul rax, rdi");
        }
        NodeKind::Div => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        _ => {}
    }

    println!("  push rax")
}

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
            error(&mut parser.lexer, msg);
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

fn error<'a>(lexer: &mut Lexer<'a>, msg: impl fmt::Display) {
    let msg = lexer.error_at(msg);
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

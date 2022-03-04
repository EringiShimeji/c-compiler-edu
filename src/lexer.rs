use std::{fmt, iter::Peekable, str::Chars, vec::IntoIter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reserved {
    LeftParen,
    RightParen,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Eq,
    Gt,
    Ge,
    Le,
    Lt,
    Ne,
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
            Reserved::Eq => "==",
            Reserved::Gt => ">",
            Reserved::Ge => ">=",
            Reserved::Le => "<=",
            Reserved::Lt => "<",
            Reserved::Ne => "!=",
        };

        write!(f, "{}", s)
    }
}

impl Reserved {
    /// 記号の長さ
    pub fn len(&self) -> usize {
        self.to_string().len()
    }
}

pub struct ReservedError(char);

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
pub enum TokenKind {
    Reserved(Reserved), // 記号
    Num(isize),         // 整数とその値
    EOF,                // 入力の終わりを表すトークン
}

#[derive(Clone)]
pub struct Token<'a> {
    kind: TokenKind,            // トークンの型
    chars: Peekable<Chars<'a>>, // そのトークン以降の文字列
}

impl<'a> Token<'a> {
    fn new(kind: TokenKind, chars: Peekable<Chars<'a>>) -> Token<'a> {
        Token { kind, chars }
    }
}

#[derive(Clone)]
pub struct Lexer<'a> {
    input: &'a String, // 入力プログラム
    chars: Peekable<Chars<'a>>,
    tokens: Peekable<IntoIter<Token<'a>>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a String) -> Lexer<'a> {
        Lexer {
            input,
            chars: input.chars().peekable(),
            tokens: vec![].into_iter().peekable(),
        }
    }

    pub fn get_input(&self) -> String {
        self.input.clone()
    }

    pub fn get_chars(&self) -> Peekable<Chars<'a>> {
        self.chars.clone()
    }

    pub fn get_tokens(&self) -> Peekable<IntoIter<Token<'a>>> {
        self.tokens.clone()
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token<'a>>, impl fmt::Display> {
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

                            self.chars.next();
                            result.push(token);
                        }
                        Err(_) => {
                            return Err("予期しない文字です");
                        }
                    }
                }
                '=' | '!' | '<' | '>' => {
                    let reserved = if self.start_with("==") {
                        Reserved::Eq
                    } else if self.start_with("!=") {
                        Reserved::Ne
                    } else if self.start_with("<=") {
                        Reserved::Le
                    } else if self.start_with(">=") {
                        Reserved::Ge
                    } else if self.start_with("<") {
                        Reserved::Lt
                    } else if self.start_with(">") {
                        Reserved::Gt
                    } else {
                        return Err("予期しない文字です");
                    };
                    let reserved_len = reserved.len();
                    let token = Token::new(TokenKind::Reserved(reserved), self.chars.clone());

                    for _ in 0..reserved_len {
                        self.chars.next();
                    }

                    result.push(token);
                }
                c if c.is_numeric() => {
                    let num = match self.take_num_str() {
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

    /// 数値であるような文字列全体もしくは先頭から続く部分列を取り出す
    /// 0から始まる数字の羅列や、数字以外が含まれる場合は、途中まで読み込んだ数値と不正な文字をタプルとしてErrで返す
    /// 備考: C言語のstrtolの仕様を参考にした
    pub fn take_num_str(&mut self) -> Result<String, (String, char)> {
        let mut result = String::new();

        while let Some(c) = self.chars.peek() {
            match c {
                // 先頭の空白は無視する
                c if result.len() == 0 && c.is_whitespace() => {
                    self.chars.next();
                }

                // 符号付き整数の可能性がある
                '+' | '-' => {
                    // 符号の位置が先頭なら、文字列全体もしくは先頭から続く部分列が整数である可能性がある
                    if result.len() == 0 {
                        let op = *c;

                        // イテレータの2番目の要素が数字かどうか調べる
                        if let Some(c) = self.chars.clone().nth(1) {
                            match c {
                                // 符号の後に数字が続けば、符号付き整数であると評価する
                                c if c.is_numeric() => {
                                    // `-0`は`0`とみなすため、先頭の符号は無視する
                                    if op == '-' && c != '0' {
                                        result.push(op);
                                    }

                                    self.chars.next();
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
                        self.chars.next();

                        if let Some(next_char) = self.chars.peek() {
                            return Err(("0".to_string(), *next_char));
                        } else {
                            return Ok("0".to_string());
                        }
                    }

                    result.push(*c);
                    self.chars.next();
                }

                c => {
                    return Err((result, *c));
                }
            }
        }

        Ok(result)
    }

    /// 与えられた文字列から始まるかどうかを判定する
    /// 元のイテレータは読み進めない
    pub fn start_with(&self, s: &'static str) -> bool {
        let mut target = self.chars.clone().take(s.len());
        let mut input = s.chars();

        while let Some(c_target) = target.next() {
            if let Some(c_input) = input.next() {
                if c_target != c_input {
                    return false;
                }
            }
        }

        match input.next() {
            Some(_) => {
                return false;
            }
            None => {
                return true;
            }
        }
    }

    pub fn at_eof(&mut self) -> bool {
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
    pub fn consume(&mut self, expect: Reserved) -> bool {
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
    pub fn expect(&mut self, expect: Reserved) -> Result<(), String> {
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
    pub fn expect_number(&mut self) -> Result<isize, String> {
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
    pub fn error_at(&mut self, msg: impl fmt::Display) -> String {
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

#[cfg(test)]
mod test {
    use super::Lexer;

    #[test]
    fn take_num_str() {
        {
            let input = "123".to_string();
            let mut lexer = Lexer::new(&input);

            assert_eq!(input, lexer.take_num_str().unwrap());
        }

        {
            let input = "+456".to_string();
            let mut lexer = Lexer::new(&input);

            assert_eq!(input[1..4], lexer.take_num_str().unwrap());
        }

        {
            let input = "-789".to_string();
            let mut lexer = Lexer::new(&input);

            assert_eq!(input, lexer.take_num_str().unwrap());
        }

        {
            let input = "0".to_string();
            let mut lexer = Lexer::new(&input);

            assert_eq!(input, lexer.take_num_str().unwrap());
        }

        {
            let input = "-0".to_string();
            let mut lexer = Lexer::new(&input);

            assert_eq!(input[1..2], lexer.take_num_str().unwrap());
        }

        {
            let input = "123a".to_string();
            let mut lexer = Lexer::new(&input);

            assert_eq!(("123".to_string(), 'a'), lexer.take_num_str().unwrap_err());
        }

        {
            let input = "5+20".to_string();
            let mut lexer = Lexer::new(&input);

            assert_eq!(("5".to_string(), '+'), lexer.take_num_str().unwrap_err());
        }
    }

    #[test]
    fn start_with() {
        let input = "hello".to_string();
        let lexer = Lexer::new(&input);

        assert_eq!(true, lexer.start_with("hello"));
        assert_eq!(true, lexer.start_with("h"));
        assert_eq!(false, lexer.start_with("adsf"));
        assert_eq!(false, lexer.start_with("ha"));
        assert_eq!(false, lexer.start_with("ha"));
        assert_eq!(false, lexer.start_with("hello world"));
    }
}

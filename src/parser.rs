use crate::lexer::{Lexer, Reserved};

/// 抽象構文木のノードの種類
#[derive(Clone, Copy)]
pub enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Num(isize),
}

/// 抽象構文木のノード
#[derive(Clone)]
pub struct Node {
    kind: NodeKind,         // ノードの型
    lhs: Option<Box<Node>>, // 左辺
    rhs: Option<Box<Node>>, // 右辺
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Node {
        Node { kind, lhs, rhs }
    }

    pub fn get_kind(&self) -> NodeKind {
        self.kind
    }

    pub fn get_lhs(&self) -> Option<Box<Node>> {
        self.lhs.clone()
    }

    pub fn get_rhs(&self) -> Option<Box<Node>> {
        self.rhs.clone()
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Parser<'a> {
        Parser { lexer }
    }

    pub fn get_lexer(&self) -> Lexer<'a> {
        self.lexer.clone()
    }

    pub fn expr(&mut self) -> Result<Node, String> {
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

    pub fn mul(&mut self) -> Result<Node, String> {
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

    pub fn primary(&mut self) -> Result<Node, String> {
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

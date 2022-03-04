use crate::lexer::{Lexer, Reserved};

/// 抽象構文木のノードの種類
#[derive(Clone, Copy)]
pub enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Le,
    Ne,
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
        self.equality()
    }

    pub fn equality(&mut self) -> Result<Node, String> {
        let mut node = match self.relational() {
            Ok(node) => node,
            Err(msg) => {
                return Err(msg);
            }
        };

        loop {
            let kind = if self.lexer.consume(Reserved::Eq) {
                NodeKind::Eq
            } else if self.lexer.consume(Reserved::Ne) {
                NodeKind::Ne
            } else {
                return Ok(node);
            };

            match self.relational() {
                Ok(relational) => {
                    node = Node::new(kind, Some(Box::new(node)), Some(Box::new(relational)));
                }
                Err(msg) => {
                    return Err(msg);
                }
            }
        }
    }

    pub fn relational(&mut self) -> Result<Node, String> {
        let mut node = match self.add() {
            Ok(node) => node,
            Err(msg) => {
                return Err(msg);
            }
        };

        loop {
            let (kind, is_reverse) = if self.lexer.consume(Reserved::Lt) {
                (NodeKind::Lt, false)
            } else if self.lexer.consume(Reserved::Le) {
                (NodeKind::Le, false)
            } else if self.lexer.consume(Reserved::Gt) {
                (NodeKind::Lt, true)
            } else if self.lexer.consume(Reserved::Ge) {
                (NodeKind::Le, true)
            } else {
                return Ok(node);
            };

            match self.add() {
                Ok(add) => {
                    if is_reverse {
                        node = Node::new(kind, Some(Box::new(add)), Some(Box::new(node)));
                    } else {
                        node = Node::new(kind, Some(Box::new(node)), Some(Box::new(add)));
                    }
                }
                Err(msg) => {
                    return Err(msg);
                }
            }
        }
    }

    pub fn add(&mut self) -> Result<Node, String> {
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
        let mut node = match self.unary() {
            Ok(node) => node,
            Err(msg) => {
                return Err(msg);
            }
        };

        loop {
            if self.lexer.consume(Reserved::Asterisk) {
                match self.unary() {
                    Ok(unary) => {
                        node =
                            Node::new(NodeKind::Mul, Some(Box::new(node)), Some(Box::new(unary)));
                    }
                    Err(msg) => {
                        return Err(msg);
                    }
                }
            } else if self.lexer.consume(Reserved::Slash) {
                match self.unary() {
                    Ok(unary) => {
                        node =
                            Node::new(NodeKind::Div, Some(Box::new(node)), Some(Box::new(unary)));
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

    pub fn unary(&mut self) -> Result<Node, String> {
        if self.lexer.consume(Reserved::Plus) {
            return self.primary();
        }

        if self.lexer.consume(Reserved::Minus) {
            match self.primary() {
                Ok(node) => {
                    return Ok(Node::new(
                        NodeKind::Sub,
                        Some(Box::new(Node::new(NodeKind::Num(0), None, None))),
                        Some(Box::new(node)),
                    ));
                }
                Err(msg) => {
                    return Err(msg);
                }
            }
        }

        return self.primary();
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

        Err("予期しないトークンです".to_string())
    }
}

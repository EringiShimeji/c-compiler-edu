use crate::parser::{Node, NodeKind};

pub fn gen(node: Node) {
    let node_kind = node.get_kind();

    if let NodeKind::Num(num) = node_kind {
        println!("  push {}", num);
        return;
    }

    if let Some(lhs) = node.get_lhs() {
        gen(*lhs);
    };

    if let Some(rhs) = node.get_rhs() {
        gen(*rhs);
    };

    println!("  pop rdi");
    println!("  pop rax");

    match node_kind {
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
        NodeKind::Eq => {
            println!("  cmp rax, rdi");
            println!("  sete al");
            println!("  movzb rax, al");
        }
        NodeKind::Ne => {
            println!("  cmp rax, rdi");
            println!("  setne al");
            println!("  movzb rax, al");
        }
        NodeKind::Lt => {
            println!("  cmp rax, rdi");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        NodeKind::Le => {
            println!("  cmp rax, rdi");
            println!("  setle al");
            println!("  movzb rax, al");
        }
        _ => {
            panic!("予期しないノードです")
        }
    }

    println!("  push rax")
}

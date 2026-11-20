//! Prints a [`Program`] as `.cp` source.

use super::ast::*;
use std::fmt::Write;

pub fn to_source(p: &Program) -> String {
    let mut out = String::new();
    for f in &p.funcs {
        let params: Vec<String> = f
            .params
            .iter()
            .map(|(n, t)| format!("{n}: {}", ty(*t)))
            .collect();
        writeln!(
            out,
            "fn {}({}) -> {}:",
            f.name,
            params.join(", "),
            ty(f.ret)
        )
        .unwrap();
        block(&mut out, &f.body, 1);
        out.push('\n');
    }
    block(&mut out, &p.body, 0);
    out
}

fn ty(t: Ty) -> &'static str {
    match t {
        Ty::Int => "Int",
        Ty::Bool => "Bool",
        Ty::List => "List<Int>",
    }
}

fn block(out: &mut String, stmts: &[Stmt], depth: usize) {
    for s in stmts {
        stmt(out, s, depth);
    }
}

fn stmt(out: &mut String, s: &Stmt, depth: usize) {
    let pad = "    ".repeat(depth);
    match s {
        Stmt::Let {
            name,
            ty: t,
            mutable,
            init,
        } => {
            let kw = if *mutable { "var" } else { "let" };
            writeln!(out, "{pad}{kw} {name}: {} = {}", ty(*t), expr(init)).unwrap();
        }
        Stmt::Assign { name, op, value } => {
            writeln!(out, "{pad}{name} {} {}", op.symbol(), expr(value)).unwrap();
        }
        Stmt::IndexAssign {
            list,
            index,
            op,
            value,
        } => {
            writeln!(
                out,
                "{pad}{list}[{}] {} {}",
                expr(index),
                op.symbol(),
                expr(value)
            )
            .unwrap();
        }
        Stmt::Push { list, value } => {
            writeln!(out, "{pad}{list}.push({})", expr(value)).unwrap();
        }
        Stmt::If { arms, els } => {
            for (i, (cond, body)) in arms.iter().enumerate() {
                let kw = if i == 0 { "if" } else { "elif" };
                writeln!(out, "{pad}{kw} {}:", expr(cond)).unwrap();
                block(out, body, depth + 1);
            }
            if let Some(body) = els {
                writeln!(out, "{pad}else:").unwrap();
                block(out, body, depth + 1);
            }
        }
        Stmt::ForRange {
            var,
            lo,
            hi,
            inclusive,
            body,
        } => {
            let dots = if *inclusive { "..=" } else { ".." };
            writeln!(out, "{pad}for {var} in {}{dots}{}:", atom(lo), atom(hi)).unwrap();
            block(out, body, depth + 1);
        }
        Stmt::ForIn { var, list, body } => {
            writeln!(out, "{pad}for {var} in {list}:").unwrap();
            block(out, body, depth + 1);
        }
        Stmt::While { cond, body } => {
            writeln!(out, "{pad}while {}:", expr(cond)).unwrap();
            block(out, body, depth + 1);
        }
        Stmt::Break => writeln!(out, "{pad}break").unwrap(),
        Stmt::Continue => writeln!(out, "{pad}continue").unwrap(),
        Stmt::Print(args) => {
            let args: Vec<String> = args.iter().map(item).collect();
            writeln!(out, "{pad}print({})", args.join(", ")).unwrap();
        }
        Stmt::Return(e) => writeln!(out, "{pad}return {}", expr(e)).unwrap(),
        Stmt::Tail(e) => writeln!(out, "{pad}{}", expr(e)).unwrap(),
    }
}

/// An expression that needs no parentheses around it as an operand.
fn atom(e: &Expr) -> String {
    match e {
        Expr::Int(v) if *v >= 0 => v.to_string(),
        Expr::Bool(_)
        | Expr::Var(_)
        | Expr::Call(..)
        | Expr::Builtin(..)
        | Expr::Index(..)
        | Expr::Len(_)
        | Expr::ListLit(_) => expr(e),
        _ => format!("({})", expr(e)),
    }
}

/// Operands are always parenthesized unless atomic, so the meaning of the
/// printed program never depends on a backend's precedence table. (A
/// precedence fuzzer would print without them; that is a separate mode.)
pub fn expr(e: &Expr) -> String {
    match e {
        Expr::Int(v) => v.to_string(),
        Expr::Bool(b) => b.to_string(),
        Expr::Var(n) => n.clone(),
        Expr::Neg(a) => format!("-{}", atom(a)),
        Expr::Not(a) => format!("not {}", atom(a)),
        Expr::Bin(op, a, b) => format!("{} {} {}", atom(a), op.symbol(), atom(b)),
        Expr::If(c, a, b) => format!("if {}: {} else: {}", expr(c), atom(a), atom(b)),
        Expr::Call(f, args) => call(f, args),
        Expr::Builtin(b, args) => {
            let name = match b {
                Builtin::Min => "min",
                Builtin::Max => "max",
                Builtin::Abs => "abs",
            };
            call(name, args)
        }
        Expr::Index(xs, i) => format!("{xs}[{}]", item(i)),
        Expr::Len(xs) => format!("{xs}.len"),
        Expr::ListLit(items) => {
            let items: Vec<String> = items.iter().map(item).collect();
            format!("[{}]", items.join(", "))
        }
    }
}

/// An expression inside brackets or an argument list: an `if` gets
/// parentheses so its `else` arm cannot swallow what follows.
fn item(e: &Expr) -> String {
    match e {
        Expr::If(..) => format!("({})", expr(e)),
        _ => expr(e),
    }
}

fn call(name: &str, args: &[Expr]) -> String {
    let args: Vec<String> = args.iter().map(item).collect();
    format!("{name}({})", args.join(", "))
}

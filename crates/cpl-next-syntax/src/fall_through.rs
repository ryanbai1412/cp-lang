//! Where running a fn's body can reach its end without giving a value (E0409): a fn that
//! gives a value gives one, or leaves, on every path through its body. Read over the
//! lossless tree; the old pipeline's `cpl_syntax::fall_through` stays frozen with it.

use crate::ast::{
    ArmBody, AstNode, Block, BlockEntry, BlockExpr, CondPart, Condition, ElseClause, Expr, FnBody,
    IfExpr, IfStmt, Stmt,
};
use crate::token::TokenKind;
use crate::tree::SyntaxNode;
use cpl_diag::Span;

/// Where the body can end without a value, and how it does, for the message.
pub type End = Option<(Span, &'static str)>;

impl FnBody {
    /// Where running the body can reach its end without giving a value.
    pub fn falls_through(&self) -> End {
        match self {
            FnBody::Block(b) => b.falls_through(),
            FnBody::Expr(e) => e.falls_through(),
        }
    }

    /// Running the body can't reach its end; see [`Block::never_ends`].
    pub fn never_ends(&self) -> bool {
        match self {
            FnBody::Block(b) => b.never_ends(),
            FnBody::Expr(e) => e.never_ends(),
        }
    }
}

impl Block {
    /// Where running the block can reach its end without giving a value.
    pub fn falls_through(&self) -> End {
        stmt_falls(&last_stmt(self.entries())?)
    }

    /// The block always leaves before its end: its last statement is a `return`, `break`,
    /// `continue` or `assert false`, or an expression that leaves.
    pub fn leaves(&self) -> bool {
        last_stmt(self.entries()).is_some_and(|s| stmt_leaves(&s))
    }

    /// The block can't reach its end: it leaves ([`Block::leaves`]), or its last statement is
    /// a `while true` with no `break` out of it.
    pub fn never_ends(&self) -> bool {
        self.leaves() || last_stmt(self.entries()).is_some_and(|s| loops_forever(&s))
    }
}

impl BlockExpr {
    /// See [`Block::falls_through`].
    pub fn falls_through(&self) -> End {
        stmt_falls(&last_stmt(self.entries())?)
    }

    /// See [`Block::leaves`].
    pub fn leaves(&self) -> bool {
        last_stmt(self.entries()).is_some_and(|s| stmt_leaves(&s))
    }

    /// See [`Block::never_ends`].
    pub fn never_ends(&self) -> bool {
        self.leaves() || last_stmt(self.entries()).is_some_and(|s| loops_forever(&s))
    }
}

impl ArmBody {
    /// See [`Block::falls_through`].
    pub fn falls_through(&self) -> End {
        match self {
            ArmBody::Block(b) => b.falls_through(),
            ArmBody::Expr(e) => e.falls_through(),
            ArmBody::Stmt(s) => stmt_falls(s),
        }
    }

    /// See [`Block::leaves`].
    pub fn leaves(&self) -> bool {
        match self {
            ArmBody::Block(b) => b.leaves(),
            ArmBody::Expr(e) => e.leaves(),
            ArmBody::Stmt(s) => stmt_leaves(s),
        }
    }
}

impl Expr {
    /// Running the expression always leaves: `todo`, a `return`, `break` or `continue` in
    /// value position, or a block, `if`/`else` or `match` all of whose paths leave.
    pub fn leaves(&self) -> bool {
        match self {
            Expr::Name(n) => n.ident().is_some_and(|t| t.text() == "todo"),
            Expr::Return(_) | Expr::Break(_) | Expr::Continue(_) => true,
            Expr::Block(b) => b.leaves(),
            Expr::If(i) => if_expr_leaves(i),
            Expr::Match(m) => {
                let mut arms = m.arms().peekable();
                arms.peek().is_some() && arms.all(|a| a.body().is_some_and(|b| b.leaves()))
            }
            _ => false,
        }
    }

    /// Running the expression can't reach its end; see [`Block::never_ends`].
    pub fn never_ends(&self) -> bool {
        match self {
            Expr::Block(b) => b.never_ends(),
            _ => self.leaves(),
        }
    }

    /// Where running the expression can end without giving a value; see
    /// [`Block::falls_through`].
    pub fn falls_through(&self) -> End {
        match self {
            Expr::Block(b) => b.falls_through(),
            Expr::If(i) => if_falls(
                i.syntax(),
                i.body()
                    .into_iter()
                    .chain(i.elifs().filter_map(|e| e.body())),
                i.else_clause(),
            ),
            Expr::Match(m) => m.arms().find_map(|a| a.body()?.falls_through()),
            _ => None,
        }
    }

    /// The first `e?` not under a `??` that would catch it, read left to right.
    pub fn bare_try(&self) -> Option<Span> {
        match self {
            Expr::Try(t) => Some(written_span(t.syntax())),
            Expr::Bin(b)
                if b.op()
                    .is_some_and(|t| t.kind() == TokenKind::QuestionQuestion) =>
            {
                None
            }
            Expr::Bin(b) => [b.lhs(), b.rhs()]
                .into_iter()
                .flatten()
                .find_map(|e| e.bare_try()),
            Expr::Compare(c) => c.operands().find_map(|e| e.bare_try()),
            Expr::Prefix(p) => p.operand()?.bare_try(),
            Expr::Paren(p) => p.inner()?.bare_try(),
            Expr::Unwrap(u) => u.inner()?.bare_try(),
            Expr::Field(f) => f.base()?.bare_try(),
            Expr::Call(c) => c
                .callee()
                .and_then(|e| e.bare_try())
                .or_else(|| c.args()?.args().find_map(|a| a.value()?.bare_try())),
            Expr::Index(i) => i
                .base()
                .and_then(|e| e.bare_try())
                .or_else(|| i.indices().find_map(|e| e.bare_try())),
            _ => None,
        }
    }
}

/// The help for a fn `name` giving `value` (with its article) whose body ends as `how` says.
pub fn help(name: &str, value: &str, how: &str) -> String {
    let fix = if how.contains("`if`") {
        format!("give `{name}` {value} on every path: an `else:` for the other case, or a value after the `if`")
    } else {
        format!("give `{name}` {value} for when the loop ends, after it")
    };
    format!("{fix}; `todo` stands for a value not written yet")
}

/// The node's written bytes: from its first token that isn't trivia to its last.
pub fn written_span(node: &SyntaxNode) -> Span {
    let mut tokens = node.tokens().filter(|t| !t.kind().is_trivia());
    let Some(first) = tokens.next() else {
        let at = node.span().start;
        return Span::new(at, at);
    };
    let last = tokens.last().unwrap_or_else(|| first.clone());
    Span::new(first.span().start, last.span().end)
}

/// The last entry of a block when it is a statement; a block ending in a local decl ends
/// with no value, as the old block's item statement did.
fn last_stmt(entries: impl Iterator<Item = BlockEntry>) -> Option<Stmt> {
    match entries.last()? {
        BlockEntry::Stmt(s) => Some(s),
        BlockEntry::Decl(_) => None,
    }
}

/// The span of the first `n` bytes of `node`'s written text, for a keyword.
fn head(node: &SyntaxNode, n: u32) -> Span {
    let at = written_span(node).start;
    Span::new(at, at + n)
}

fn stmt_falls(s: &Stmt) -> End {
    match s {
        Stmt::Expr(e) => e.expr()?.falls_through(),
        Stmt::If(i) => if_falls(
            i.syntax(),
            i.body()
                .map(ArmBody::Block)
                .into_iter()
                .chain(i.elifs().filter_map(|e| e.body())),
            i.else_clause(),
        ),
        Stmt::Unchecked(u) => u.body()?.falls_through(),
        Stmt::While(w) => loop_falls(
            head(w.syntax(), 5),
            forever(w.cond()),
            w.body(),
            w.else_clause(),
        ),
        Stmt::For(f) => loop_falls(head(f.syntax(), 3), false, f.body(), f.else_clause()),
        _ => None,
    }
}

fn stmt_leaves(s: &Stmt) -> bool {
    match s {
        Stmt::Return(_) | Stmt::Break(_) | Stmt::Continue(_) => true,
        Stmt::Assert(a) => a.cond().is_some_and(|c| is_bool(&c, false)),
        Stmt::Unchecked(u) => u.body().is_some_and(|b| b.leaves()),
        Stmt::Expr(e) => e.expr().is_some_and(|e| e.leaves()),
        Stmt::If(i) => if_stmt_leaves(i),
        _ => false,
    }
}

fn loops_forever(s: &Stmt) -> bool {
    matches!(s, Stmt::While(w) if forever(w.cond())
        && !w.body().is_some_and(|b| breaks_out(&b, false, &mut Vec::new())))
}

fn if_falls(
    node: &SyntaxNode,
    mut branches: impl Iterator<Item = ArmBody>,
    otherwise: Option<ElseClause>,
) -> End {
    if let Some(end) = branches.find_map(|b| b.falls_through()) {
        return Some(end);
    }
    match otherwise {
        Some(e) => e.body()?.falls_through(),
        None => Some((head(node, 2), "this `if` has no `else`")),
    }
}

fn if_stmt_leaves(i: &IfStmt) -> bool {
    i.else_clause()
        .and_then(|e| e.body())
        .is_some_and(|b| b.leaves())
        && i.body().is_some_and(|b| b.leaves())
        && i.elifs().all(|e| e.body().is_some_and(|b| b.leaves()))
}

fn if_expr_leaves(i: &IfExpr) -> bool {
    i.else_clause()
        .and_then(|e| e.body())
        .is_some_and(|b| b.leaves())
        && i.body().is_some_and(|b| b.leaves())
        && i.elifs().all(|e| e.body().is_some_and(|b| b.leaves()))
}

fn is_bool(e: &Expr, value: bool) -> bool {
    let want = if value {
        TokenKind::True
    } else {
        TokenKind::False
    };
    matches!(e, Expr::Literal(l) if l.token().is_some_and(|t| t.kind() == want))
}

/// `while true`; `while True:` already has its error, so it reads as the `true` it means.
fn forever(cond: Option<Condition>) -> bool {
    let Some(cond) = cond else { return false };
    let mut parts = cond.parts();
    let (Some(CondPart::Expr(e)), None) = (parts.next(), parts.next()) else {
        return false;
    };
    match &e {
        Expr::Name(n) => n.ident().is_some_and(|t| {
            cpl_diag::habits::find(cpl_diag::habits::Kind::Value, t.text())
                .is_some_and(|h| h.fix == "true")
        }),
        _ => is_bool(&e, true),
    }
}

/// A loop ends unless it is `while true` with no `break` out of it; after
/// an `else:` the loop's end is the `else` block's.
fn loop_falls(head: Span, forever: bool, body: Option<Block>, else_: Option<ElseClause>) -> End {
    let breaks = body.is_some_and(|b| breaks_out(&b, false, &mut Vec::new()));
    match else_.and_then(|e| e.block()) {
        _ if forever && !breaks => None,
        Some(b) if !breaks => b.falls_through(),
        _ => Some((head, "this loop can end")),
    }
}

/// Whether a `break` in `block` leaves the loop it is the body of:
/// `nested` inside an inner loop only a labelled `break` past the
/// labels in `inner` does.
fn breaks_out(block: &Block, nested: bool, inner: &mut Vec<String>) -> bool {
    block.stmts().any(|s| stmt_breaks(&s, nested, inner))
}

fn stmt_breaks(s: &Stmt, nested: bool, inner: &mut Vec<String>) -> bool {
    let in_loop = |label: Option<String>,
                   body: Option<Block>,
                   else_: Option<ElseClause>,
                   inner: &mut Vec<String>| {
        let n = inner.len();
        inner.extend(label);
        let out = body.is_some_and(|b| breaks_out(&b, true, inner));
        inner.truncate(n);
        out || else_
            .and_then(|e| e.block())
            .is_some_and(|b| breaks_out(&b, nested, inner))
    };
    match s {
        Stmt::Break(b) => match b.label() {
            None => !nested,
            Some(l) => !inner.iter().any(|i| i == l.text()),
        },
        Stmt::While(w) => in_loop(
            w.label().map(|l| l.text().to_owned()),
            w.body(),
            w.else_clause(),
            inner,
        ),
        Stmt::For(f) => in_loop(
            f.label().map(|l| l.text().to_owned()),
            f.body(),
            f.else_clause(),
            inner,
        ),
        Stmt::Unchecked(u) => u.body().is_some_and(|b| breaks_out(&b, nested, inner)),
        Stmt::Expr(e) => e.expr().is_some_and(|e| expr_breaks(&e, nested, inner)),
        Stmt::If(i) => {
            i.body().is_some_and(|b| breaks_out(&b, nested, inner))
                || i.elifs()
                    .filter_map(|e| e.body())
                    .chain(i.else_clause().and_then(|e| e.body()))
                    .any(|b| arm_breaks(&b, nested, inner))
        }
        _ => false,
    }
}

fn arm_breaks(b: &ArmBody, nested: bool, inner: &mut Vec<String>) -> bool {
    match b {
        ArmBody::Block(b) => breaks_out(b, nested, inner),
        ArmBody::Expr(e) => expr_breaks(e, nested, inner),
        ArmBody::Stmt(s) => stmt_breaks(s, nested, inner),
    }
}

fn expr_breaks(e: &Expr, nested: bool, inner: &mut Vec<String>) -> bool {
    match e {
        Expr::Break(b) => match b.label() {
            None => !nested,
            Some(l) => !inner.iter().any(|i| i == l.text()),
        },
        Expr::Block(b) => b.stmts().any(|s| stmt_breaks(&s, nested, inner)),
        Expr::If(i) => i
            .body()
            .into_iter()
            .chain(i.elifs().filter_map(|e| e.body()))
            .chain(i.else_clause().and_then(|e| e.body()))
            .any(|b| arm_breaks(&b, nested, inner)),
        Expr::Match(m) => m
            .arms()
            .filter_map(|a| a.body())
            .any(|b| arm_breaks(&b, nested, inner)),
        _ => false,
    }
}

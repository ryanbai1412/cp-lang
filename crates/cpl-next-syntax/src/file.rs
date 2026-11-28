//! A file's tree: its items, the trivia between them and its end.

use crate::items::split_items;
use crate::lexer::lex;
use crate::token::{Token, TokenKind};
use crate::tree::{GreenBuilder, SyntaxNode};
use cpl_diag::Diagnostic;

/// The kinds of node in the tree.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NodeKind {
    /// The whole file: its items, the trivia between them and `Eof`.
    File,
    /// One top-level item (`split_items`): the comment lines directly above
    /// it, then its first token to its last.
    Item,
    /// Bytes the parser skipped. Already reported.
    Error,
    /// A required child that was not written: zero width, already reported.
    Missing,
    /// A number, string, raw string or char, `true` or `false`.
    Literal,
    /// A name used as a value: `x`, `self`.
    NameRef,
    /// `(e)`.
    ParenExpr,
    /// `()`, `(a, b)`, `(a,)`.
    TupleExpr,
    /// `[a, b]`.
    ListExpr,
    /// `-x`, `not x`, `~x`.
    PrefixExpr,
    /// `a + b`, with the operator from `ops::BINARY`.
    BinExpr,
    /// `a < b <= c`: one comparison chain.
    CompareExpr,
    /// `lo..hi`, `..=hi`, `lo..`, `lo..hi by s`.
    RangeExpr,
    /// `f(args)`.
    CallExpr,
    /// `(a, name: b)` after a callee.
    ArgList,
    /// One argument: `e`, `name: e`, `mut e`.
    Arg,
    /// `a[i, j]`.
    IndexExpr,
    /// `[i, j]` after an indexed value.
    IndexList,
    /// `a.f`, `t.0`.
    FieldExpr,
    /// `x?`.
    TryExpr,
    /// `x!`.
    UnwrapExpr,
    /// `x is P`.
    IsExpr,
    /// `best max= x`.
    KeepExpr,
    /// `x => e`, `(a, b) => e`.
    LambdaExpr,
    /// A lambda's parameters: one pattern, or `(p, q: T)`.
    ParamList,
    /// `_`.
    WildcardPat,
    /// `x`, `None`, `Some(v)`, `Color.Red`, `E::V(a)`.
    PathPat,
    /// `(p, q)` after a variant path.
    PatArgs,
    /// `(p, q)`.
    TuplePat,
    /// `1`, `-1`, `"s"`.
    LiteralPat,
    /// `p | q`.
    OrPat,
    /// `: stmts` on its line, or `:` then an indented run of statements.
    Block,
    /// `let p: T = e else: ...`, also `var` and `const`.
    LetStmt,
    ExprStmt,
    /// `place = e`, `place += e`, `place f= e`.
    AssignStmt,
    ReturnStmt,
    BreakStmt,
    ContinueStmt,
    YieldStmt,
    /// `assert c, message`.
    AssertStmt,
    /// `if c: ... elif c: ... else: ...`.
    IfStmt,
    /// `if c: a elif d: b else: e` as a value.
    IfExpr,
    ElifClause,
    ElseClause,
    /// A condition: comma-separated tests and `let p = e` bindings.
    Condition,
    /// `let p = e` inside a condition.
    LetCond,
    WhileStmt,
    ForStmt,
    /// `p in xs if f`.
    ForClause,
    /// `match e:` and its arms, as a statement or a value.
    MatchExpr,
    /// `p if g => body`.
    MatchArm,
    /// `with ctx(..): ...`.
    WithStmt,
    UncheckedStmt,
    /// `repeat k: ...`.
    RepeatStmt,
    /// `Name`, `a.B`, `Vec<Int>`.
    PathType,
    /// `<T, 3>` after a type's name.
    GenericArgs,
    /// `?T`.
    OptionalType,
    /// `[D]T`, `[n, m]T`.
    ArrayType,
    /// `[D]` before an array's element type.
    Dims,
    /// `(A, B)`, `()`.
    TupleType,
    /// `(lo: Int, hi: V, ..R)`.
    RecordType,
    /// `name: T` in a record type.
    RecordTypeField,
    /// `(A, B) -> C`.
    FnType,
    /// `0..n`, `..=9` as a type.
    RangeType,
    /// `1 | 2 | 3`, `"a" | "b"`.
    LiteralType,
}

/// One file's syntax.
#[derive(Clone, Debug)]
pub struct SyntaxFile {
    pub root: SyntaxNode,
    /// What was wrong with the file's text, in file positions.
    pub diagnostics: Vec<Diagnostic>,
}

impl SyntaxFile {
    /// The file's items, in order.
    pub fn items(&self) -> impl Iterator<Item = SyntaxNode> + '_ {
        self.root.children().filter(|n| n.kind() == NodeKind::Item)
    }
}

/// Where the comments that belong to the next item start in the trivia
/// before it: the comment lines directly above it, with no blank line
/// between them or below the last. They move and format with the item, and
/// its docs are read from them.
fn attached_comments(src: &str, before: &[&Token]) -> usize {
    let mut k = before.len();
    while k >= 2
        && before[k - 1].kind == TokenKind::Whitespace
        && before[k - 1].span.text(src).matches('\n').count() == 1
        && before[k - 2].kind == TokenKind::Comment
    {
        k -= 2;
    }
    k
}

/// The tree of `src`. Total: any text gives a tree whose text is `src`.
pub fn parse(src: &str) -> SyntaxFile {
    let lexed = lex(src);
    let items = split_items(&lexed.tokens);
    let tokens = lexed.lossless(src);
    let mut b = GreenBuilder::default();
    b.start_node(NodeKind::File);
    let push = |b: &mut GreenBuilder, t: &Token| b.token(t.kind, t.span.text(src));
    let mut all = tokens.iter().peekable();
    // Index of the next non-trivia token, as `split_items` counts them.
    let mut next = 0usize;
    for item in &items {
        let (first, last) = item.tokens;
        let mut before = Vec::new();
        while let Some(t) = all.next_if(|t| next < first || t.kind.is_trivia()) {
            next += usize::from(!t.kind.is_trivia());
            before.push(t);
        }
        let docs = attached_comments(src, &before);
        for t in &before[..docs] {
            push(&mut b, t);
        }
        b.start_node(NodeKind::Item);
        for t in &before[docs..] {
            push(&mut b, t);
        }
        while let Some(t) = all.next_if(|_| next < last) {
            next += usize::from(!t.kind.is_trivia());
            push(&mut b, t);
        }
        b.finish_node();
    }
    for t in all {
        push(&mut b, t);
    }
    b.finish_node();
    SyntaxFile {
        root: SyntaxNode::new_root(b.finish()),
        diagnostics: lexed.diagnostics,
    }
}

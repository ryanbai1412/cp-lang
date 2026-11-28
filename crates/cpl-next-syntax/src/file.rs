//! A file's tree: its items, the trivia between them and its end.

use crate::ast::{self, AstNode};
use crate::items::split_items;
use crate::lexer::lex;
use crate::parser::parse_item;
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
    /// A name a declaration introduces: a fn's, a type's, a field's, a
    /// parameter's. Holds the `Ident`, or a `Missing` node.
    Name,
    /// A number, string, raw string or char, `true` or `false`.
    Literal,
    /// A name used as a value: `x`, `self`.
    /// `_` where a value goes: an end of `_..=_`.
    HoleExpr,
    NameRef,
    /// `(e)`.
    ParenExpr,
    /// `()`, `(a, b)`, `(a,)`.
    TupleExpr,
    /// `[a, b]`.
    ListExpr,
    /// `[k: v, ...]`, `[:]`.
    MapExpr,
    /// `k: v` in a map.
    MapEntry,
    /// `[body for p in xs if c]`.
    CompExpr,
    /// `for p in xs if c` in a comprehension or a generator.
    CompClause,
    /// `f(x) for x in xs`, a call's argument.
    GeneratorExpr,
    /// `[D, E] (d, e) => body`.
    TableExpr,
    /// `[n] value`: a table of `n` cells, each `value`.
    TableFill,
    /// `(name: e, ...)`.
    RecordExpr,
    /// `name: e`, or `name:` for the variable `name`.
    RecordField,
    /// `..e` or `..e without (u, v)` in a record expression.
    RecordRest,
    /// `Name<T>`: a name with type arguments, as a value.
    GenericExpr,
    /// `+` where a value goes: the fn `(a, b) => a + b`.
    OpValue,
    /// `@intrinsic("name")`, `@cpp("code")`.
    IntrinsicExpr,
    /// `{ s; s; value }`.
    BlockExpr,
    /// `.0`, `.name`: the fn that reads that field.
    FieldFn,
    /// `mut a[i]`: a place, named for writing (R05.3.4).
    MutExpr,
    /// `lend mut place`, `lend place`: a `lends` body's place; code after
    /// it is write-back (R05.6.3).
    LendStmt,
    /// `T: Ord` in an `if`: true where `T` meets the bound (R08 §190).
    BoundCond,
    /// `x with f: v, g: w`: `x` with those fields replaced.
    WithExpr,
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
    /// `(x: a, y)`: a record's fields, by name.
    RecordPat,
    /// `x: a`, or `y:` for `y: y`.
    RecordPatField,
    /// `..` in `(u:, ..)`: the fields a record pattern leaves out.
    RecordPatRest,
    /// `1`, `-1`, `"s"`.
    LiteralPat,
    /// `p | q`.
    OrPat,
    /// `: stmts` on its line, or `:` then an indented run of statements.
    Block,
    /// `let p: T = e else: ...`, also `var`.
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
    /// `@trusted: ...` (R09.5.4).
    TrustedStmt,
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
    /// `uses io, rng` after a fn's signature.
    UsesClause,
    /// `(a: T, b: U)` after a struct's name.
    FieldList,
    /// `Str of "ab"`.
    AlphabetType,
    /// `domain I` in a type: a domain the value brings with it.
    DomainBinder,
    /// `0..n`, `..=9` as a type.
    RangeType,
    /// `1 | 2 | 3`, `"a" | "b"`.
    LiteralType,
    /// `_` as a type: the checker infers it.
    InferType,
    /// `type_of(f)`, `type_of(+)`: the type of a fn or an operator.
    TypeOfType,
    /// `@!name(args)` on its own line.
    Directive,
    /// `@name(args)` before an item; part of that item.
    Decorator,
    /// `fn name<G>(params) -> R where ..: body`, or a signature without one.
    FnDecl,
    /// `(a: T) + (b: T)`: an operator fn's name.
    OpFnName,
    /// `name: T = default`, `while ok: K`, `..rest: T`, `self`.
    Param,
    /// `-> yields T`, `-> lends (a) T`.
    RetType,
    /// `(a, b)` after `lends`.
    LendSrc,
    /// `<T: Bound = D, const N: Int>`.
    GenericParams,
    GenericParam,
    /// `A + B`.
    Bound,
    /// `where T: A, U: B`.
    WhereClause,
    /// `T: A` in a where clause.
    WherePred,
    StructDecl,
    /// `pub var name: T = e`.
    FieldDecl,
    EnumDecl,
    /// `Name(fields)` in an enum.
    Variant,
    TraitDecl,
    /// `type Name: Bound` in a trait, `type Name = T` in an impl.
    AssocType,
    ImplDecl,
    /// `type Name<G> = T`.
    TypeAlias,
    /// `domain N = e`, `let (domain N, ..) = e`.
    DomainDecl,
    /// `pub const N: T = e`, at the top or in a body.
    ConstDecl,
    /// `context io:` and its fields.
    ContextDecl,
    /// `mod m:` and its items, each an `Item` as at the top level.
    ModDecl,
    /// `extern fn name(params) -> R`.
    ExternDecl,
    /// `use a.b.{c, d}`.
    UseDecl,
    /// `{c, d}` in a use.
    UseNames,
    /// `law name(params) where ..= condition` in a trait or impl.
    LawDecl,
    /// An item's indented members: fields, variants, fns, laws.
    Members,
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

    /// The module's decorators, such as `@role(prelude)` (R11.6): those at
    /// the file's top, above its first `use`. A `use` takes no decorator of
    /// its own, so the ones it holds are the module's.
    pub fn decorators(&self) -> Vec<ast::Decorator> {
        let first = self.items().next().and_then(ast::Item::cast);
        match first.and_then(|i| i.decl()) {
            Some(ast::Decl::Use(u)) => u.decorators().collect(),
            _ => Vec::new(),
        }
    }
}

/// Where the comments that belong to the next item start in the trivia
/// before it: the comment lines directly above it, with no blank line
/// between them or below the last. They move and format with the item, and
/// its docs are read from them.
fn attached_comments(src: &str, before: &[Token]) -> usize {
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
/// Each item is parsed on its own (`split_items`), and the file holds the
/// trivia between items.
pub fn parse(src: &str) -> SyntaxFile {
    let lexed = lex(src);
    let items = split_items(&lexed.tokens);
    let tokens = lexed.lossless(src);
    let mut diagnostics = lexed.diagnostics;
    let mut b = GreenBuilder::default();
    b.start_node(NodeKind::File);
    let push = |b: &mut GreenBuilder, t: &Token| b.token(t.kind, t.span.text(src));
    // `at` indexes the lossless tokens; `next` counts the significant ones
    // before it, as `split_items` does.
    let (mut at, mut next) = (0usize, 0usize);
    let mut advance = |at: &mut usize, until: &dyn Fn(usize, &Token) -> bool| {
        while *at < tokens.len() && !until(next, &tokens[*at]) {
            next += usize::from(!tokens[*at].kind.is_trivia());
            *at += 1;
        }
    };
    for item in &items {
        let (first, last) = item.tokens;
        let start = at;
        advance(&mut at, &|next, t| next >= first && !t.kind.is_trivia());
        let docs = start + attached_comments(src, &tokens[start..at]);
        for t in &tokens[start..docs] {
            push(&mut b, t);
        }
        advance(&mut at, &|next, _| next >= last);
        let (green, errors) = parse_item(src, &tokens[docs..at]);
        b.node(green);
        diagnostics.extend(errors);
    }
    for t in &tokens[at..] {
        push(&mut b, t);
    }
    b.finish_node();
    SyntaxFile {
        root: SyntaxNode::new_root(b.finish()),
        diagnostics,
    }
}

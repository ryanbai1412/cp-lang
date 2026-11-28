//! Typed views over the tree, rust-analyzer style. A view is a node of
//! one kind, and an accessor returns `None` when the child it reads is
//! missing or broken, so a parse error never reads as a valid node. A
//! category (`Decl`, `Stmt`, `Type`, `Expr`, `Pattern`) is an enum over
//! the views of its kinds.

use crate::file::NodeKind;
use crate::grammar::{contextual, Contextual};
use crate::literal::{self, FmtSpec};
use crate::ops::{self, BinOp, CmpOp, UnOp};
use crate::token::TokenKind;
use crate::tree::{SyntaxElement, SyntaxNode, SyntaxToken};
use cpl_diag::Span;

pub trait AstNode: Sized {
    fn cast(node: SyntaxNode) -> Option<Self>;
    fn syntax(&self) -> &SyntaxNode;
}

macro_rules! nodes {
    ($($(#[$doc:meta])* $name:ident),* $(,)?) => {$(
        $(#[$doc])*
        #[derive(Clone, Debug)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            fn cast(node: SyntaxNode) -> Option<Self> {
                (node.kind() == NodeKind::$name).then_some(Self(node))
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    )*};
}

macro_rules! category {
    ($(#[$doc:meta])* $name:ident { $($variant:ident($node:ident)),* $(,)? }) => {
        $(#[$doc])*
        #[derive(Clone, Debug)]
        pub enum $name {
            $($variant($node)),*
        }

        impl AstNode for $name {
            fn cast(node: SyntaxNode) -> Option<Self> {
                match node.kind() {
                    $(NodeKind::$node => Some(Self::$variant($node(node))),)*
                    _ => None,
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                match self {
                    $(Self::$variant(n) => n.syntax(),)*
                }
            }
        }
    };
}

nodes! {
    /// One top-level item: its attached comments, then a declaration or a
    /// statement.
    Item,
    /// `@name(args)` on the line above a declaration, field or variant.
    Decorator,
    /// `@!multitest`, `@!interactive(...)`: a directive heading the item
    /// below it, or misplaced in a block.
    Directive,
    ArgList,
    /// `label: value`, `mut place` or a value.
    Arg,
    FnDecl,
    /// `fn (a: T) + (b: T)`: the name of an operator fn.
    OpFnName,
    ParamList,
    Param,
    /// `-> T`, `-> yields T`, `-> lends(a) mut T`.
    RetType,
    /// `(a, b)` in `lends(a, b)`.
    LendSrc,
    /// `uses io, rng`.
    UsesClause,
    GenericParams,
    /// `T: Bound`, `const N: Int`, `domain D`, `U = Int`.
    GenericParam,
    /// `A + B<T>`: a bound, a supertrait list, or an impl's trait.
    Bound,
    WhereClause,
    WherePred,
    StructDecl,
    /// A struct's header fields, `struct P(x: Int, y: Int)`.
    FieldList,
    FieldDecl,
    EnumDecl,
    Variant,
    TraitDecl,
    ImplDecl,
    /// The members of a struct, enum, trait, impl or context.
    Members,
    TypeAlias,
    /// `type Elem` in a trait, `type Elem = Int` in an impl.
    AssocType,
    DomainDecl,
    ConstDecl,
    ContextDecl,
    ModDecl,
    ExternDecl,
    UseDecl,
    UseNames,
    LawDecl,
    Block,
    ElseClause,
    LetStmt,
    ExprStmt,
    AssignStmt,
    ReturnStmt,
    BreakStmt,
    ContinueStmt,
    YieldStmt,
    AssertStmt,
    IfStmt,
    WhileStmt,
    ForStmt,
    WithStmt,
    UncheckedStmt,
    TrustedStmt,
    RepeatStmt,
    LendStmt,
    /// `Int`, `List<Int>`, `std.Pair`, `Str(len: 1..=n)`.
    PathType,
    GenericArgs,
    OptionalType,
    /// `[V]Int`, `[Row, Col]Char`, `[domain S]Char`.
    ArrayType,
    Dims,
    /// `(A, B)`, `(mut S, T)`.
    TupleType,
    /// `(lo: Int, hi: Int, ..R)`.
    RecordType,
    RecordTypeField,
    FnType,
    /// `Str of "01"`.
    AlphabetType,
    /// `domain P`, in a tuple or record type.
    DomainBinder,
    /// A range `0..=1e9` or a value `n - 1` in a type.
    RangeType,
    /// `'+' | '-'`, `1 | 2`, `7`.
    LiteralType,
    InferType,
    TypeOfType,
    Literal,
    /// `{x}` or `{x:.2f}` in a string `Literal`: the value and its format.
    Interp,
    NameRef,
    HoleExpr,
    ParenExpr,
    TupleExpr,
    ListExpr,
    MapExpr,
    CompExpr,
    GeneratorExpr,
    TableExpr,
    RecordExpr,
    /// `x: 1` or a punned `x:` in a record expression.
    RecordField,
    /// `..e` or `..e without (u, v)` in a record expression.
    RecordRest,
    GenericExpr,
    OpValue,
    IntrinsicExpr,
    BlockExpr,
    FieldFn,
    MutExpr,
    WithExpr,
    PrefixExpr,
    BinExpr,
    CompareExpr,
    RangeExpr,
    CallExpr,
    IndexExpr,
    FieldExpr,
    TryExpr,
    UnwrapExpr,
    IsExpr,
    KeepExpr,
    LambdaExpr,
    IfExpr,
    MatchExpr,
    WildcardPat,
    /// `x`, `None`, `Some(v)`, `Op.Add(l, r)`.
    PathPat,
    PatArgs,
    TuplePat,
    RecordPat,
    RecordPatField,
    OrPat,
    LiteralPat,
    ParenPat,
    MapEntry,
    CompClause,
    TableFill,
    IndexList,
    Condition,
    LetCond,
    BoundCond,
    ElifClause,
    ForClause,
    MatchArm,
}

category! {
    /// A declaration an item can hold.
    Decl {
        Fn(FnDecl),
        Struct(StructDecl),
        Enum(EnumDecl),
        Trait(TraitDecl),
        Impl(ImplDecl),
        TypeAlias(TypeAlias),
        Domain(DomainDecl),
        Const(ConstDecl),
        Context(ContextDecl),
        Mod(ModDecl),
        Extern(ExternDecl),
        Use(UseDecl),
    }
}

category! {
    Stmt {
        Let(LetStmt),
        Expr(ExprStmt),
        Assign(AssignStmt),
        Return(ReturnStmt),
        Break(BreakStmt),
        Continue(ContinueStmt),
        Yield(YieldStmt),
        Assert(AssertStmt),
        If(IfStmt),
        While(WhileStmt),
        For(ForStmt),
        With(WithStmt),
        Unchecked(UncheckedStmt),
        Trusted(TrustedStmt),
        Repeat(RepeatStmt),
        Lend(LendStmt),
    }
}

category! {
    Type {
        Path(PathType),
        Optional(OptionalType),
        Array(ArrayType),
        Tuple(TupleType),
        Record(RecordType),
        Fn(FnType),
        Alphabet(AlphabetType),
        DomainBinder(DomainBinder),
        Range(RangeType),
        Literal(LiteralType),
        Infer(InferType),
        TypeOf(TypeOfType),
    }
}

category! {
    Expr {
        Literal(Literal),
        Name(NameRef),
        Hole(HoleExpr),
        Paren(ParenExpr),
        Tuple(TupleExpr),
        List(ListExpr),
        Map(MapExpr),
        Comp(CompExpr),
        Generator(GeneratorExpr),
        Table(TableExpr),
        Record(RecordExpr),
        Generic(GenericExpr),
        OpValue(OpValue),
        Intrinsic(IntrinsicExpr),
        Block(BlockExpr),
        FieldFn(FieldFn),
        Mut(MutExpr),
        With(WithExpr),
        Prefix(PrefixExpr),
        Bin(BinExpr),
        Compare(CompareExpr),
        Range(RangeExpr),
        Call(CallExpr),
        Index(IndexExpr),
        Field(FieldExpr),
        Try(TryExpr),
        Unwrap(UnwrapExpr),
        Is(IsExpr),
        Keep(KeepExpr),
        Lambda(LambdaExpr),
        If(IfExpr),
        Match(MatchExpr),
        Fill(TableFill),
        // `return`, `break` and `continue` where a value goes, as in
        // `else: return None`: a value of type `Never`.
        Return(ReturnStmt),
        Break(BreakStmt),
        Continue(ContinueStmt),
    }
}

category! {
    Pattern {
        Wildcard(WildcardPat),
        Path(PathPat),
        Tuple(TuplePat),
        Paren(ParenPat),
        Record(RecordPat),
        Or(OrPat),
        Literal(LiteralPat),
    }
}

/// A type argument or a dimension: a type (a value is a `RangeType` or
/// `LiteralType`), or `K: Ord` naming a parameter in an impl's head.
#[derive(Clone, Debug)]
pub enum GenericArg {
    Type(Type),
    Param(GenericParam),
}

impl AstNode for GenericArg {
    fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == NodeKind::GenericParam {
            return Some(Self::Param(GenericParam(node)));
        }
        Type::cast(node).map(Self::Type)
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Type(n) => n.syntax(),
            Self::Param(n) => n.syntax(),
        }
    }
}

/// A fn's body: a block, or the value after `=`.
#[derive(Clone, Debug)]
pub enum FnBody {
    Block(Block),
    Expr(Expr),
}

fn child<N: AstNode>(node: &SyntaxNode) -> Option<N> {
    node.children().find_map(N::cast)
}

fn children<'a, N: AstNode + 'a>(node: &'a SyntaxNode) -> impl Iterator<Item = N> + 'a {
    node.children().filter_map(N::cast)
}

/// `node`'s own tokens, trivia left out.
fn own_tokens(node: &SyntaxNode) -> impl Iterator<Item = SyntaxToken> + '_ {
    node.children_with_tokens().filter_map(|e| match e {
        SyntaxElement::Token(t) if !t.kind().is_trivia() => Some(t),
        _ => None,
    })
}

fn own_token(node: &SyntaxNode, kind: TokenKind) -> Option<SyntaxToken> {
    own_tokens(node).find(|t| t.kind() == kind)
}

fn has(node: &SyntaxNode, kind: TokenKind) -> bool {
    own_token(node, kind).is_some()
}

fn is_word(t: &SyntaxToken, word: Contextual) -> bool {
    t.kind() == TokenKind::Ident && contextual(t.text()) == Some(word)
}

/// The first `N` child after `node`'s own `kind` token.
fn after<N: AstNode>(node: &SyntaxNode, kind: TokenKind) -> Option<N> {
    let at = own_token(node, kind)?.span().end;
    node.children()
        .filter(|n| n.span().start >= at)
        .find_map(N::cast)
}

/// The first `N` child before `node`'s own `kind` token, or anywhere when
/// there is no such token.
fn before<N: AstNode>(node: &SyntaxNode, kind: TokenKind) -> Option<N> {
    let end = own_token(node, kind).map_or(u32::MAX, |t| t.span().start);
    node.children()
        .take_while(|n| n.span().start < end)
        .find_map(N::cast)
}

/// The `Ident` of `node`'s `Name` child: `None` when the name is missing.
fn name_of(node: &SyntaxNode) -> Option<SyntaxToken> {
    names_in(node).next()
}

/// The `Ident` of each of `node`'s `Name` children.
fn names_in(node: &SyntaxNode) -> impl Iterator<Item = SyntaxToken> + '_ {
    node.children()
        .filter(|n| n.kind() == NodeKind::Name)
        .filter_map(|n| n.tokens().find(|t| t.kind() == TokenKind::Ident))
}

macro_rules! named {
    ($($node:ident),*) => {$(
        impl $node {
            /// The declared name; `None` when it is missing.
            pub fn name(&self) -> Option<SyntaxToken> {
                name_of(&self.0)
            }
        }
    )*};
}

macro_rules! visible {
    ($($node:ident),*) => {$(
        impl $node {
            pub fn is_pub(&self) -> bool {
                has(&self.0, TokenKind::Pub)
            }
        }
    )*};
}

macro_rules! generic {
    ($($node:ident),*) => {$(
        impl $node {
            pub fn generics(&self) -> Option<GenericParams> {
                child(&self.0)
            }
        }
    )*};
}

macro_rules! decorated {
    ($($node:ident),*) => {$(
        impl $node {
            pub fn decorators(&self) -> impl Iterator<Item = Decorator> + '_ {
                children(&self.0)
            }
        }
    )*};
}

named!(
    FnDecl,
    StructDecl,
    EnumDecl,
    TraitDecl,
    TypeAlias,
    AssocType,
    DomainDecl,
    ConstDecl,
    ContextDecl,
    ModDecl,
    ExternDecl,
    LawDecl,
    Variant,
    FieldDecl,
    GenericParam,
    DomainBinder
);
visible!(
    FnDecl,
    StructDecl,
    EnumDecl,
    TraitDecl,
    TypeAlias,
    AssocType,
    ConstDecl,
    ContextDecl,
    ModDecl,
    UseDecl,
    FieldDecl
);
generic!(FnDecl, StructDecl, EnumDecl, TraitDecl, ImplDecl, TypeAlias, AssocType);
decorated!(
    FnDecl,
    StructDecl,
    EnumDecl,
    TraitDecl,
    ImplDecl,
    TypeAlias,
    DomainDecl,
    ConstDecl,
    ContextDecl,
    ModDecl,
    ExternDecl,
    UseDecl,
    Variant,
    FieldDecl
);

impl Item {
    /// The declaration the item holds; `None` for a statement or a
    /// directive.
    pub fn decl(&self) -> Option<Decl> {
        child(&self.0)
    }

    /// The statement the item is, at the top level or in a mod.
    pub fn stmt(&self) -> Option<Stmt> {
        child(&self.0)
    }

    /// The name the item declares. An impl, an operator fn, a statement
    /// and a broken declaration have none; a `use` has its `as` name.
    pub fn name(&self) -> Option<SyntaxToken> {
        name_of(self.decl()?.syntax())
    }

    /// A declaration holds its decorators; a decorated statement's sit in
    /// the item.
    pub fn decorators(&self) -> Vec<Decorator> {
        let holder = self
            .decl()
            .map_or_else(|| self.0.clone(), |d| d.syntax().clone());
        children(&holder).collect()
    }

    pub fn is_pub(&self) -> bool {
        self.decl().is_some_and(|d| has(d.syntax(), TokenKind::Pub))
    }

    /// The `@!name` heading the item.
    pub fn directive(&self) -> Option<Directive> {
        child(&self.0)
    }
}

impl Directive {
    /// `multitest` in `@!multitest`.
    pub fn name(&self) -> Option<SyntaxToken> {
        own_tokens(&self.0).find(|t| t.kind() == TokenKind::Ident)
    }

    pub fn args(&self) -> Option<ArgList> {
        child(&self.0)
    }
}

impl Decorator {
    /// `cache` in `@cache`; `std` in `@std.tag(1)`.
    pub fn name(&self) -> Option<SyntaxToken> {
        self.path().next()
    }

    /// The dotted name's segments.
    pub fn path(&self) -> impl Iterator<Item = SyntaxToken> + '_ {
        own_tokens(&self.0).filter(|t| t.kind() == TokenKind::Ident)
    }

    pub fn args(&self) -> Option<ArgList> {
        child(&self.0)
    }
}

impl ArgList {
    pub fn args(&self) -> impl Iterator<Item = Arg> + '_ {
        children(&self.0)
    }
}

impl Arg {
    /// `by` in `f(by: 2)`, a reserved word or a name.
    pub fn label(&self) -> Option<SyntaxToken> {
        let tokens: Vec<_> = own_tokens(&self.0).collect();
        let colon = tokens.iter().position(|t| t.kind() == TokenKind::Colon)?;
        tokens.get(colon.checked_sub(1)?).cloned()
    }

    pub fn is_mut(&self) -> bool {
        has(&self.0, TokenKind::Mut)
    }

    /// `None` for `f(by:)`, which passes the variable the label names.
    pub fn value(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl FnDecl {
    pub fn op_name(&self) -> Option<OpFnName> {
        child(&self.0)
    }

    /// The parameters; an operator fn's are in its [`OpFnName`].
    pub fn params(&self) -> Option<ParamList> {
        child(&self.0)
    }

    pub fn ret(&self) -> Option<RetType> {
        child(&self.0)
    }

    pub fn uses(&self) -> Option<UsesClause> {
        child(&self.0)
    }

    pub fn where_clause(&self) -> Option<WhereClause> {
        child(&self.0)
    }

    /// `None` for a fn with no body, as in a trait.
    pub fn body(&self) -> Option<FnBody> {
        child(&self.0)
            .map(FnBody::Block)
            .or_else(|| child(&self.0).map(FnBody::Expr))
    }

    /// From `pub` or `fn` to the end of the where clause.
    pub fn signature_span(&self) -> Span {
        let start = own_tokens(&self.0)
            .next()
            .map_or(self.0.span().start, |t| t.span().start);
        let head = self.0.children().filter(|n| {
            matches!(
                n.kind(),
                NodeKind::Name
                    | NodeKind::OpFnName
                    | NodeKind::GenericParams
                    | NodeKind::ParamList
                    | NodeKind::RetType
                    | NodeKind::UsesClause
                    | NodeKind::WhereClause
            )
        });
        Span::new(start, head.last().map_or(start, |n| n.span().end))
    }
}

impl OpFnName {
    /// The operator: `+` in `fn (a: P) + (b: P)`, `-` in `fn -(a: P)`.
    pub fn op(&self) -> Option<SyntaxToken> {
        own_tokens(&self.0).find(|t| {
            let k = t.kind();
            ops::binary(k).is_some() || ops::prefix(k).is_some() || k == TokenKind::In
        })
    }

    /// `fn -(a: P)`: the operator comes before its one operand.
    pub fn is_prefix(&self) -> bool {
        match (self.op(), self.params().next()) {
            (Some(op), Some(lhs)) => op.span().start < lhs.syntax().span().start,
            _ => false,
        }
    }

    /// The operands' parameter lists, left then right (one for a prefix).
    pub fn params(&self) -> impl Iterator<Item = ParamList> + '_ {
        children(&self.0)
    }
}

impl ParamList {
    pub fn params(&self) -> impl Iterator<Item = Param> + '_ {
        children(&self.0)
    }
}

impl Param {
    /// `mut x: T`: lends the caller's place (R05.2.3).
    pub fn is_mut(&self) -> bool {
        has(&self.0, TokenKind::Mut)
    }

    /// `var x: T`: owns its argument (R05.2.3).
    pub fn is_var(&self) -> bool {
        has(&self.0, TokenKind::Var)
    }

    pub fn is_variadic(&self) -> bool {
        has(&self.0, TokenKind::DotDot)
    }

    pub fn is_self(&self) -> bool {
        has(&self.0, TokenKind::SelfKw)
    }

    /// After the `;` in its list: passed by label only.
    pub fn is_named_only(&self) -> bool {
        let Some(list) = self.0.parent() else {
            return false;
        };
        own_token(&list, TokenKind::Semi).is_some_and(|semi| semi.span().end <= self.0.span().start)
    }

    /// The outer label in `while ok: K`, a reserved word or a name.
    pub fn label(&self) -> Option<SyntaxToken> {
        let end = self.pattern()?.syntax().span().start;
        own_tokens(&self.0)
            .take_while(|t| t.span().start < end)
            .filter(|t| !matches!(t.kind(), TokenKind::Mut | TokenKind::Var))
            .last()
    }

    pub fn pattern(&self) -> Option<Pattern> {
        child(&self.0)
    }

    /// `x` in `x: T`, `..xs` or `self`; `None` for a destructuring pattern.
    pub fn name(&self) -> Option<SyntaxToken> {
        name_of(&self.0)
            .or_else(|| own_token(&self.0, TokenKind::SelfKw))
            .or_else(|| match self.pattern()? {
                Pattern::Path(p) => p.binding(),
                _ => None,
            })
    }

    pub fn ty(&self) -> Option<Type> {
        child(&self.0)
    }

    pub fn default(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl RetType {
    /// `-> yields T`: a generator.
    pub fn is_yields(&self) -> bool {
        own_tokens(&self.0).any(|t| is_word(&t, Contextual::Yields))
    }

    pub fn lends(&self) -> bool {
        has(&self.0, TokenKind::Lends)
    }

    /// `-> lends mut T`.
    pub fn lends_mut(&self) -> bool {
        has(&self.0, TokenKind::Mut)
    }

    pub fn lend_sources(&self) -> Option<LendSrc> {
        child(&self.0)
    }

    pub fn ty(&self) -> Option<Type> {
        child(&self.0)
    }
}

impl LendSrc {
    pub fn names(&self) -> impl Iterator<Item = SyntaxToken> + '_ {
        names_in(&self.0)
    }
}

impl UsesClause {
    pub fn contexts(&self) -> impl Iterator<Item = SyntaxToken> + '_ {
        own_tokens(&self.0)
            .filter(|t| t.kind() == TokenKind::Ident)
            .skip(1)
    }
}

impl GenericParams {
    pub fn params(&self) -> impl Iterator<Item = GenericParam> + '_ {
        children(&self.0)
    }
}

impl GenericParam {
    pub fn is_const(&self) -> bool {
        has(&self.0, TokenKind::Const)
    }

    pub fn is_domain(&self) -> bool {
        has(&self.0, TokenKind::Domain)
    }

    pub fn bound(&self) -> Option<Bound> {
        child(&self.0)
    }

    /// `Int` in `const N: Int`.
    pub fn ty(&self) -> Option<Type> {
        before(&self.0, TokenKind::Eq)
    }

    /// `Int` in `U = Int`.
    pub fn default(&self) -> Option<Type> {
        after(&self.0, TokenKind::Eq)
    }
}

impl Bound {
    pub fn types(&self) -> impl Iterator<Item = Type> + '_ {
        children(&self.0)
    }
}

impl WhereClause {
    pub fn preds(&self) -> impl Iterator<Item = WherePred> + '_ {
        children(&self.0)
    }
}

impl WherePred {
    pub fn ty(&self) -> Option<Type> {
        child(&self.0)
    }

    pub fn bound(&self) -> Option<Bound> {
        child(&self.0)
    }
}

impl StructDecl {
    /// Header fields, then fields after a one-line `:`, then body fields.
    pub fn fields(&self) -> Vec<FieldDecl> {
        let mut fields: Vec<FieldDecl> = child::<FieldList>(&self.0)
            .map(|l| children(&l.0).collect())
            .unwrap_or_default();
        fields.extend(children(&self.0));
        fields.extend(self.members().iter().flat_map(Members::fields));
        fields
    }

    pub fn members(&self) -> Option<Members> {
        child(&self.0)
    }
}

impl FieldDecl {
    pub fn is_var(&self) -> bool {
        has(&self.0, TokenKind::Var)
    }

    /// `None` for `var top = 0`, whose default gives the type.
    pub fn ty(&self) -> Option<Type> {
        child(&self.0)
    }

    pub fn default(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl EnumDecl {
    /// One-line variants, then body variants.
    pub fn variants(&self) -> Vec<Variant> {
        let mut variants: Vec<Variant> = children(&self.0).collect();
        variants.extend(self.members().iter().flat_map(Members::variants));
        variants
    }

    pub fn members(&self) -> Option<Members> {
        child(&self.0)
    }
}

impl Variant {
    pub fn fields(&self) -> impl Iterator<Item = FieldDecl> + '_ {
        children(&self.0)
    }
}

impl TraitDecl {
    /// `A + B` in `trait Both = A + B`.
    pub fn alias(&self) -> Option<Bound> {
        after(&self.0, TokenKind::Eq)
    }

    /// `Eq` in `trait Ord(Eq)` or `trait Ord: Eq`.
    pub fn supertraits(&self) -> Option<Bound> {
        (!has(&self.0, TokenKind::Eq))
            .then(|| child(&self.0))
            .flatten()
    }

    pub fn where_clause(&self) -> Option<WhereClause> {
        child(&self.0)
    }

    /// Fields after a one-line `:`, then body fields.
    pub fn fields(&self) -> Vec<FieldDecl> {
        let mut fields: Vec<FieldDecl> = children(&self.0).collect();
        fields.extend(self.members().iter().flat_map(Members::fields));
        fields
    }

    pub fn members(&self) -> Option<Members> {
        child(&self.0)
    }
}

impl ImplDecl {
    /// The trait in `impl Trait for T`; `None` for an inherent `impl T`.
    pub fn trait_ref(&self) -> Option<Bound> {
        self.has_for().then(|| child(&self.0)).flatten()
    }

    /// The type the impl is for: `T` in `impl T` and `impl Trait for T`.
    pub fn self_ty(&self) -> Option<Type> {
        if self.has_for() {
            return child(&self.0);
        }
        let mut types = child::<Bound>(&self.0)?.types().collect::<Vec<_>>();
        (types.len() == 1).then(|| types.pop()).flatten()
    }

    pub fn where_clause(&self) -> Option<WhereClause> {
        child(&self.0)
    }

    pub fn members(&self) -> Option<Members> {
        child(&self.0)
    }

    fn has_for(&self) -> bool {
        has(&self.0, TokenKind::For)
    }
}

impl Members {
    pub fn fns(&self) -> impl Iterator<Item = FnDecl> + '_ {
        children(&self.0)
    }

    pub fn laws(&self) -> impl Iterator<Item = LawDecl> + '_ {
        children(&self.0)
    }

    pub fn fields(&self) -> impl Iterator<Item = FieldDecl> + '_ {
        children(&self.0)
    }

    pub fn variants(&self) -> impl Iterator<Item = Variant> + '_ {
        children(&self.0)
    }

    pub fn assoc_types(&self) -> impl Iterator<Item = AssocType> + '_ {
        children(&self.0)
    }

    pub fn consts(&self) -> impl Iterator<Item = ConstDecl> + '_ {
        children(&self.0)
    }

    /// An impl's `let count = e`.
    pub fn lets(&self) -> impl Iterator<Item = LetStmt> + '_ {
        children(&self.0)
    }

    /// A struct's `domain Idx = 0..src.len`.
    pub fn domains(&self) -> impl Iterator<Item = DomainDecl> + '_ {
        children(&self.0)
    }
}

impl TypeAlias {
    pub fn bound(&self) -> Option<Bound> {
        child(&self.0)
    }

    pub fn ty(&self) -> Option<Type> {
        child(&self.0)
    }
}

impl AssocType {
    pub fn bound(&self) -> Option<Bound> {
        child(&self.0)
    }

    /// `None` for a trait's `type Elem`.
    pub fn ty(&self) -> Option<Type> {
        child(&self.0)
    }
}

impl ConstDecl {
    pub fn ty(&self) -> Option<Type> {
        child(&self.0)
    }

    pub fn value(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl DomainDecl {
    /// `e` in `domain N = e` and `let (domain N, i) = e`.
    pub fn size(&self) -> Option<Expr> {
        child(&self.0)
    }

    /// `i` in `let (domain N, i) = e`.
    pub fn patterns(&self) -> impl Iterator<Item = Pattern> + '_ {
        children(&self.0)
    }
}

impl ContextDecl {
    pub fn fields(&self) -> Vec<FieldDecl> {
        self.members().iter().flat_map(Members::fields).collect()
    }

    pub fn members(&self) -> Option<Members> {
        child(&self.0)
    }
}

impl ModDecl {
    /// The mod's items, each read like a top-level item.
    pub fn items(&self) -> impl Iterator<Item = Item> + '_ {
        children(&self.0)
    }
}

impl ExternDecl {
    pub fn params(&self) -> Option<ParamList> {
        child(&self.0)
    }

    pub fn ret(&self) -> Option<RetType> {
        child(&self.0)
    }
}

impl UseDecl {
    /// `lib.treap` in `use lib.treap.{Treap, Node}`.
    pub fn path(&self) -> impl Iterator<Item = SyntaxToken> + '_ {
        own_tokens(&self.0)
            .take_while(|t| !is_word(t, Contextual::As))
            .filter(|t| t.kind() == TokenKind::Ident)
    }

    /// `Treap, Node` in `use lib.treap.{Treap, Node}`.
    pub fn names(&self) -> Option<UseNames> {
        child(&self.0)
    }

    /// `use m.*`.
    pub fn is_glob(&self) -> bool {
        has(&self.0, TokenKind::Star)
    }

    /// `fl` in `use lib.graph.flow as fl`.
    pub fn alias(&self) -> Option<SyntaxToken> {
        name_of(&self.0)
    }
}

impl UseNames {
    pub fn names(&self) -> impl Iterator<Item = SyntaxToken> + '_ {
        own_tokens(&self.0).filter(|t| t.kind() == TokenKind::Ident)
    }
}

impl LawDecl {
    pub fn params(&self) -> Option<ParamList> {
        child(&self.0)
    }

    pub fn where_clause(&self) -> Option<WhereClause> {
        child(&self.0)
    }

    pub fn body(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl Block {
    pub fn stmts(&self) -> impl Iterator<Item = Stmt> + '_ {
        children(&self.0)
    }

    /// Its statements and local items (a fn, `domain N = e`), as written.
    pub fn entries(&self) -> impl Iterator<Item = BlockEntry> + '_ {
        children(&self.0)
    }

    /// The `@!name` lines written in the block, which belong above an item.
    pub fn directives(&self) -> impl Iterator<Item = Directive> + '_ {
        children(&self.0)
    }
}

impl ElseClause {
    pub fn block(&self) -> Option<Block> {
        child(&self.0)
    }

    /// The branch in an `if` value: a value on the line or a block.
    pub fn body(&self) -> Option<ArmBody> {
        child(&self.0)
    }
}

impl LetStmt {
    /// `a` and `b` in `let a, b = e`.
    pub fn patterns(&self) -> impl Iterator<Item = Pattern> + '_ {
        children(&self.0)
    }

    pub fn ty(&self) -> Option<Type> {
        child(&self.0)
    }

    pub fn value(&self) -> Option<Expr> {
        child(&self.0)
    }

    /// `var x = e`: the bindings are `var` (R01.6).
    pub fn is_var(&self) -> bool {
        has(&self.0, TokenKind::Var)
    }

    /// `let Some(x) = e else: ...`.
    pub fn else_clause(&self) -> Option<ElseClause> {
        child(&self.0)
    }
}

impl PathType {
    /// `std`, `Pair` in `std.Pair<A, B>`; `Self` is a segment too.
    pub fn segments(&self) -> impl Iterator<Item = SyntaxToken> + '_ {
        own_tokens(&self.0).filter(|t| matches!(t.kind(), TokenKind::Ident | TokenKind::SelfKw))
    }

    pub fn generic_args(&self) -> Option<GenericArgs> {
        child(&self.0)
    }

    /// `(len: 1..=n)` in `Str(len: 1..=n)`.
    pub fn args(&self) -> Option<ArgList> {
        child(&self.0)
    }
}

impl GenericArgs {
    pub fn args(&self) -> impl Iterator<Item = GenericArg> + '_ {
        children(&self.0)
    }
}

impl OptionalType {
    pub fn inner(&self) -> Option<Type> {
        child(&self.0)
    }

    /// `?mut T`: an optional lent-`mut` place.
    pub fn is_mut(&self) -> bool {
        has(&self.0, TokenKind::Mut)
    }
}

impl ArrayType {
    pub fn dims(&self) -> Option<Dims> {
        child(&self.0)
    }

    pub fn elem(&self) -> Option<Type> {
        child(&self.0)
    }
}

impl Dims {
    /// A domain type, a `domain S` binder, or a value.
    pub fn args(&self) -> impl Iterator<Item = GenericArg> + '_ {
        children(&self.0)
    }
}

impl TupleType {
    /// `(T)`: one type in parentheses with no comma, which is `T` itself; only a fn type's
    /// parameter list reads it as one parameter.
    pub fn is_grouping(&self) -> bool {
        self.elems().len() == 1 && !has(&self.0, TokenKind::Comma) && !has(&self.0, TokenKind::Mut)
    }

    /// Each element's type and whether it is `mut`, as a fn type's
    /// parameters can be.
    pub fn elems(&self) -> Vec<(bool, Type)> {
        let mut mutable = false;
        let mut elems = Vec::new();
        for e in self.0.children_with_tokens() {
            match e {
                SyntaxElement::Token(t) if t.kind() == TokenKind::Mut => mutable = true,
                SyntaxElement::Node(n) => {
                    if let Some(ty) = Type::cast(n) {
                        elems.push((mutable, ty));
                    }
                    mutable = false;
                }
                SyntaxElement::Token(_) => {}
            }
        }
        elems
    }
}

impl RecordType {
    pub fn fields(&self) -> impl Iterator<Item = RecordTypeField> + '_ {
        children(&self.0)
    }
}

impl RecordExpr {
    pub fn fields(&self) -> impl Iterator<Item = RecordField> + '_ {
        children(&self.0)
    }

    /// `..e` in `(..e, to: u)` or `(to: v, ..e without (u, v))`.
    pub fn rest(&self) -> Option<RecordRest> {
        child(&self.0)
    }

    /// `(..e, to: u)`, a record update like `e with to: u` (R02.5.3): the rest comes first and
    /// drops nothing, so it is no `without` form.
    pub fn is_update(&self) -> bool {
        self.rest().is_some_and(|r| {
            let at = r.syntax().span().start;
            r.removed().is_empty() && self.fields().all(|f| f.syntax().span().start > at)
        })
    }
}

impl RecordField {
    pub fn name(&self) -> Option<SyntaxToken> {
        own_token(&self.0, TokenKind::Ident)
    }

    /// `None` for a punned `x:`, which is `x: x`.
    pub fn value(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl RecordRest {
    /// `e` in `..e`: the record the other fields come from.
    pub fn expr(&self) -> Option<Expr> {
        child(&self.0)
    }

    /// `u`, `v` in `..e without (u, v)`: the fields the rest drops.
    pub fn removed(&self) -> Vec<SyntaxToken> {
        own_tokens(&self.0)
            .skip_while(|t| !is_word(t, Contextual::Without))
            .skip(1)
            .filter(|t| t.kind() == TokenKind::Ident)
            .collect()
    }
}

impl RecordTypeField {
    /// `None` for a rest `..R`.
    pub fn name(&self) -> Option<SyntaxToken> {
        if self.is_rest() {
            return None;
        }
        own_token(&self.0, TokenKind::Ident)
    }

    /// `u`, `v` in `..R without (u, v)`: the fields the rest drops.
    pub fn removed(&self) -> Vec<SyntaxToken> {
        own_tokens(&self.0)
            .skip_while(|t| !is_word(t, Contextual::Without))
            .skip(1)
            .filter(|t| t.kind() == TokenKind::Ident)
            .collect()
    }

    /// `..` or `..R`: the open rest of a row.
    pub fn is_rest(&self) -> bool {
        has(&self.0, TokenKind::DotDot)
    }

    pub fn ty(&self) -> Option<Type> {
        child(&self.0)
    }
}

impl FnType {
    /// The parameter type: a `TupleType` for `(A, B) -> C`.
    pub fn param(&self) -> Option<Type> {
        children(&self.0).next()
    }

    /// `-> yields S` and `-> lends T` read as a fn's result does.
    pub fn ret(&self) -> Option<RetType> {
        child(&self.0)
    }
}

impl AlphabetType {
    pub fn base(&self) -> Option<Type> {
        child(&self.0)
    }

    /// The `"01"` in `Str of "01"`.
    pub fn alphabet(&self) -> Option<SyntaxToken> {
        own_token(&self.0, TokenKind::Str)
    }
}

impl RangeType {
    /// A `RangeExpr` for a range, any other expression for a value.
    pub fn expr(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl TypeOfType {
    /// `f` or `+` in `type_of(f)`, `type_of(+)`.
    pub fn arg(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl LiteralType {
    pub fn literals(&self) -> impl Iterator<Item = SyntaxToken> + '_ {
        own_tokens(&self.0).filter(|t| t.kind() != TokenKind::Pipe)
    }
}

impl Literal {
    pub fn token(&self) -> Option<SyntaxToken> {
        own_tokens(&self.0).next()
    }

    /// A string's pieces in written order, escapes decoded; `None` when the
    /// literal is not a string. A raw string is one piece, as written.
    pub fn pieces(&self) -> Option<Vec<StrPiece>> {
        let first = self.token()?;
        match first.kind() {
            TokenKind::RawStr => {
                let text = &first.text()[2..];
                let inner = text.strip_suffix('"').unwrap_or(text);
                Some(vec![StrPiece::Text(inner.to_string())])
            }
            TokenKind::Str | TokenKind::StrText => {
                let text = self.0.text();
                let mut holes = children::<Interp>(&self.0);
                let pieces = literal::string_parts(&text).into_iter().map(|p| match p {
                    literal::StrPart::Text(s) => {
                        Some(StrPiece::Text(literal::unescape(s.text(&text))))
                    }
                    literal::StrPart::Expr(_) => holes.next().map(StrPiece::Interp),
                });
                pieces.collect()
            }
            _ => None,
        }
    }
}

/// One piece of a string literal.
#[derive(Clone, Debug)]
pub enum StrPiece {
    Text(String),
    Interp(Interp),
}

impl Interp {
    pub fn value(&self) -> Option<Expr> {
        child(&self.0)
    }

    /// The format after the `:` (`{x:05}`), when it spells one; the parser
    /// reported any other.
    pub fn format(&self) -> Option<FmtSpec> {
        own_tokens(&self.0)
            .find(|t| t.kind() == TokenKind::StrText)
            .and_then(|t| FmtSpec::parse(t.text()))
    }
}

impl NameRef {
    pub fn ident(&self) -> Option<SyntaxToken> {
        own_tokens(&self.0).next()
    }

    /// `None`, the empty optional: a literal word the parser reads as a name.
    pub fn is_none(&self) -> bool {
        self.ident().is_some_and(|t| {
            crate::token::LiteralWord::of(t.text()) == Some(crate::token::LiteralWord::None)
        })
    }
}

impl ParenExpr {
    pub fn inner(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl FieldExpr {
    pub fn base(&self) -> Option<Expr> {
        child(&self.0)
    }

    /// The name after `.`: a word, a reserved word or a tuple index.
    pub fn name(&self) -> Option<SyntaxToken> {
        own_tokens(&self.0).nth(1)
    }
}

impl BinExpr {
    pub fn lhs(&self) -> Option<Expr> {
        children(&self.0).next()
    }

    pub fn op(&self) -> Option<SyntaxToken> {
        own_tokens(&self.0).next()
    }

    pub fn rhs(&self) -> Option<Expr> {
        children(&self.0).nth(1)
    }

    /// What the operator computes; `None` for a comparison, which is a [`CompareExpr`].
    pub fn op_kind(&self) -> Option<BinOp> {
        match ops::binary(self.op()?.kind())?.kind {
            ops::Kind::Bin(op) => Some(op),
            ops::Kind::Cmp(_) => None,
        }
    }
}

impl PrefixExpr {
    pub fn op(&self) -> Option<SyntaxToken> {
        own_tokens(&self.0).next()
    }

    pub fn operand(&self) -> Option<Expr> {
        child(&self.0)
    }

    pub fn op_kind(&self) -> Option<UnOp> {
        ops::prefix_op(self.op()?.kind())
    }
}

impl RangeExpr {
    /// `None` for `..hi`.
    pub fn lo(&self) -> Option<Expr> {
        before(&self.0, self.op()?.kind())
    }

    /// `None` for `lo..`; `hi`, not the step, in `lo..hi by s`.
    pub fn hi(&self) -> Option<Expr> {
        let at = self.op()?.span().end;
        let by = self.by().map_or(u32::MAX, |t| t.span().start);
        self.0
            .children()
            .filter(|n| n.span().start >= at && n.span().start < by)
            .find_map(Expr::cast)
    }

    /// `s` in `lo..hi by s` and in `w by s`.
    pub fn step(&self) -> Option<Expr> {
        let at = self.by()?.span().end;
        self.0
            .children()
            .filter(|n| n.span().start >= at)
            .find_map(Expr::cast)
    }

    /// `w` in `w by s`, the range written with no `..`.
    pub fn walk(&self) -> Option<Expr> {
        if self.op().is_some() {
            return None;
        }
        let by = self.by()?.span().start;
        self.0
            .children()
            .take_while(|n| n.span().start < by)
            .find_map(Expr::cast)
    }

    fn by(&self) -> Option<SyntaxToken> {
        own_tokens(&self.0).find(|t| is_word(t, Contextual::By))
    }

    pub fn is_inclusive(&self) -> bool {
        has(&self.0, TokenKind::DotDotEq)
    }

    fn op(&self) -> Option<SyntaxToken> {
        own_tokens(&self.0).find(|t| matches!(t.kind(), TokenKind::DotDot | TokenKind::DotDotEq))
    }
}

impl CallExpr {
    pub fn callee(&self) -> Option<Expr> {
        child(&self.0)
    }

    pub fn args(&self) -> Option<ArgList> {
        child(&self.0)
    }
}

impl GenericExpr {
    pub fn base(&self) -> Option<Expr> {
        child(&self.0)
    }

    pub fn args(&self) -> Option<GenericArgs> {
        child(&self.0)
    }
}

impl PathPat {
    /// `self` is a segment too: the pattern `self` binds it.
    pub fn segments(&self) -> impl Iterator<Item = SyntaxToken> + '_ {
        own_tokens(&self.0).filter(|t| matches!(t.kind(), TokenKind::Ident | TokenKind::SelfKw))
    }

    pub fn args(&self) -> Option<PatArgs> {
        child(&self.0)
    }

    /// `var x`: the binding is a `var` (R01.6).
    pub fn is_var(&self) -> bool {
        has(&self.0, TokenKind::Var)
    }

    /// The name a one-word pattern with no arguments binds or names.
    pub fn binding(&self) -> Option<SyntaxToken> {
        let mut segments = self.segments();
        let first = segments.next()?;
        (segments.next().is_none() && self.args().is_none()).then_some(first)
    }
}

impl PatArgs {
    pub fn patterns(&self) -> impl Iterator<Item = Pattern> + '_ {
        children(&self.0)
    }
}

impl TrustedStmt {
    pub fn body(&self) -> Option<Block> {
        child(&self.0)
    }
}

impl RecordPat {
    pub fn fields(&self) -> impl Iterator<Item = RecordPatField> + '_ {
        children(&self.0)
    }

    /// `(u:, ..)`: the fields not named are left out (R02.5.5).
    pub fn has_rest(&self) -> bool {
        self.0
            .children()
            .any(|n| n.kind() == NodeKind::RecordPatRest)
    }
}

impl RecordPatField {
    pub fn name(&self) -> Option<SyntaxToken> {
        own_token(&self.0, TokenKind::Ident)
    }

    /// `None` for `u:`, which binds `u`.
    pub fn pattern(&self) -> Option<Pattern> {
        child(&self.0)
    }
}

impl IntrinsicExpr {
    /// The string literal naming the intrinsic: `"dom_count"` in
    /// `@intrinsic("dom_count")`.
    pub fn id(&self) -> Option<SyntaxToken> {
        own_token(&self.0, TokenKind::Str)
    }
}

/// The body of a match arm, a lambda, a table, or an `if` value's branch:
/// a value, a statement on the line (`x = v`, `assert c`), or an indented
/// block.
#[derive(Clone, Debug)]
pub enum ArmBody {
    Block(Block),
    Expr(Expr),
    Stmt(Stmt),
}

impl AstNode for ArmBody {
    fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == NodeKind::Block {
            return Some(Self::Block(Block(node)));
        }
        match Expr::cast(node.clone()) {
            Some(e) => Some(Self::Expr(e)),
            None => Stmt::cast(node).map(Self::Stmt),
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Block(n) => n.syntax(),
            Self::Expr(n) => n.syntax(),
            Self::Stmt(n) => n.syntax(),
        }
    }
}

/// One entry of a block: a statement, or a declaration local to it (a
/// fn, `domain N = e`).
#[derive(Clone, Debug)]
pub enum BlockEntry {
    Stmt(Stmt),
    Decl(Decl),
}

impl AstNode for BlockEntry {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match Decl::cast(node.clone()) {
            Some(d) => Some(Self::Decl(d)),
            None => Stmt::cast(node).map(Self::Stmt),
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Stmt(n) => n.syntax(),
            Self::Decl(n) => n.syntax(),
        }
    }
}

/// One part of a [`Condition`]: a `Bool`, `let p = e`, or `T: Bound`.
#[derive(Clone, Debug)]
pub enum CondPart {
    Expr(Expr),
    Let(LetCond),
    Bound(BoundCond),
}

impl AstNode for CondPart {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            NodeKind::LetCond => Some(Self::Let(LetCond(node))),
            NodeKind::BoundCond => Some(Self::Bound(BoundCond(node))),
            _ => Expr::cast(node).map(Self::Expr),
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Expr(n) => n.syntax(),
            Self::Let(n) => n.syntax(),
            Self::Bound(n) => n.syntax(),
        }
    }
}

/// One parameter of a lambda or a table: `mut p: T`, the type optional.
#[derive(Clone, Debug)]
pub struct LambdaParam {
    pub is_mut: bool,
    pub pattern: Pattern,
    pub ty: Option<Type>,
}

/// The token of a loop's or a `break`'s `'label`.
fn label_of(node: &SyntaxNode) -> Option<SyntaxToken> {
    own_token(node, TokenKind::Label)
}

impl ExprStmt {
    pub fn expr(&self) -> Option<Expr> {
        child(&self.0)
    }
}

/// How an assignment writes its place: `=`, `+=`, or `p f= e` (`p = f(p, e)`).
#[derive(Clone, Debug)]
pub enum AssignOp {
    Set,
    Op(BinOp),
    /// The fn's name token: `gcd` in `g gcd= x`.
    Fn(SyntaxToken),
}

impl AssignOp {
    /// The assignment written `text`: `=` or a compound `+=`. A `p f= e` names its fn, so
    /// it has no fixed spelling.
    pub fn from_spelling(text: &str) -> Option<AssignOp> {
        match text {
            "=" => Some(AssignOp::Set),
            _ => ops::compound(text).map(AssignOp::Op),
        }
    }
}

impl std::fmt::Display for AssignOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignOp::Set => f.write_str("="),
            AssignOp::Op(op) => write!(f, "{op}="),
            AssignOp::Fn(name) => write!(f, "{}=", name.text()),
        }
    }
}

impl AssignStmt {
    pub fn op(&self) -> Option<AssignOp> {
        let mut tokens = own_tokens(&self.0).peekable();
        while let Some(t) = tokens.next() {
            match t.kind() {
                TokenKind::OpEq => return ops::compound(t.text()).map(AssignOp::Op),
                TokenKind::Ident if tokens.peek().is_some_and(|n| n.kind() == TokenKind::Eq) => {
                    return Some(AssignOp::Fn(t));
                }
                TokenKind::Eq => return Some(AssignOp::Set),
                _ => {}
            }
        }
        None
    }

    /// The place written: `a[i]` in `a[i] += 1`.
    pub fn target(&self) -> Option<Expr> {
        child(&self.0)
    }

    /// The value after the operator.
    pub fn value(&self) -> Option<Expr> {
        let op =
            own_tokens(&self.0).find(|t| matches!(t.kind(), TokenKind::Eq | TokenKind::OpEq))?;
        let at = op.span().end;
        self.0
            .children()
            .filter(|n| n.span().start >= at)
            .find_map(Expr::cast)
    }
}

impl ReturnStmt {
    /// `None` for a bare `return`.
    pub fn value(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl YieldStmt {
    pub fn value(&self) -> Option<Expr> {
        child(&self.0)
    }

    /// `yield from xs`: each value of `xs`.
    pub fn is_from(&self) -> bool {
        own_tokens(&self.0).any(|t| is_word(&t, Contextual::From))
    }
}

impl BreakStmt {
    pub fn label(&self) -> Option<SyntaxToken> {
        label_of(&self.0)
    }
}

impl ContinueStmt {
    pub fn label(&self) -> Option<SyntaxToken> {
        label_of(&self.0)
    }
}

impl AssertStmt {
    pub fn cond(&self) -> Option<Expr> {
        before(&self.0, TokenKind::Comma)
    }

    /// The message after `,`.
    pub fn message(&self) -> Option<Expr> {
        after(&self.0, TokenKind::Comma)
    }
}

impl LendStmt {
    /// `lend mut place`.
    pub fn is_mut(&self) -> bool {
        has(&self.0, TokenKind::Mut)
    }

    pub fn place(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl Condition {
    pub fn parts(&self) -> impl Iterator<Item = CondPart> + '_ {
        children(&self.0)
    }
}

impl LetCond {
    pub fn pattern(&self) -> Option<Pattern> {
        child(&self.0)
    }

    pub fn value(&self) -> Option<Expr> {
        after(&self.0, TokenKind::Eq)
    }
}

impl BoundCond {
    /// The type tested: `T` in `T: Ord`, read as a type.
    pub fn subject(&self) -> Option<Type> {
        child(&self.0)
    }

    pub fn bound(&self) -> Option<Bound> {
        child(&self.0)
    }
}

impl IfStmt {
    /// `if const c:`, chosen at compile time (R08.2.9).
    pub fn is_const(&self) -> bool {
        has(&self.0, TokenKind::Const)
    }

    pub fn cond(&self) -> Option<Condition> {
        child(&self.0)
    }

    pub fn body(&self) -> Option<Block> {
        child(&self.0)
    }

    pub fn elifs(&self) -> impl Iterator<Item = ElifClause> + '_ {
        children(&self.0)
    }

    pub fn else_clause(&self) -> Option<ElseClause> {
        child(&self.0)
    }
}

impl IfExpr {
    /// `if const c: a else: b`, chosen at compile time (R08.2.9).
    pub fn is_const(&self) -> bool {
        has(&self.0, TokenKind::Const)
    }

    pub fn cond(&self) -> Option<Condition> {
        child(&self.0)
    }

    /// The first branch: a value on the line or an indented block.
    pub fn body(&self) -> Option<ArmBody> {
        child(&self.0)
    }

    pub fn elifs(&self) -> impl Iterator<Item = ElifClause> + '_ {
        children(&self.0)
    }

    pub fn else_clause(&self) -> Option<ElseClause> {
        child(&self.0)
    }
}

impl ElifClause {
    pub fn cond(&self) -> Option<Condition> {
        child(&self.0)
    }

    /// The branch: a block in a statement, a value or a block in a value.
    pub fn body(&self) -> Option<ArmBody> {
        child(&self.0)
    }
}

impl WhileStmt {
    pub fn label(&self) -> Option<SyntaxToken> {
        label_of(&self.0)
    }

    /// `None` for an error-recovered `loop:`.
    pub fn cond(&self) -> Option<Condition> {
        child(&self.0)
    }

    pub fn body(&self) -> Option<Block> {
        child(&self.0)
    }

    /// `else:` after the loop: run when it ends without `break`.
    pub fn else_clause(&self) -> Option<ElseClause> {
        child(&self.0)
    }
}

impl ForStmt {
    pub fn label(&self) -> Option<SyntaxToken> {
        label_of(&self.0)
    }

    /// `x in xs if f, y in ys`: one clause per `in`.
    pub fn clauses(&self) -> impl Iterator<Item = ForClause> + '_ {
        children(&self.0)
    }

    pub fn body(&self) -> Option<Block> {
        child(&self.0)
    }

    pub fn else_clause(&self) -> Option<ElseClause> {
        child(&self.0)
    }
}

macro_rules! clause {
    ($($node:ident),*) => {$(
        impl $node {
            pub fn pattern(&self) -> Option<Pattern> {
                child(&self.0)
            }

            /// What is walked: the value after `in`.
            pub fn iter(&self) -> Option<Expr> {
                after(&self.0, TokenKind::In)
            }

            /// One condition per `if`.
            pub fn filters(&self) -> impl Iterator<Item = Condition> + '_ {
                children(&self.0)
            }
        }
    )*};
}

clause!(ForClause, CompClause);

impl RepeatStmt {
    /// How many times: `k` in `repeat k:`.
    pub fn count(&self) -> Option<Expr> {
        child(&self.0)
    }

    pub fn body(&self) -> Option<Block> {
        child(&self.0)
    }
}

impl UncheckedStmt {
    pub fn body(&self) -> Option<Block> {
        child(&self.0)
    }
}

impl WithStmt {
    /// The contexts installed: `io(base: 1)` in `with io(base: 1):`.
    pub fn contexts(&self) -> impl Iterator<Item = Expr> + '_ {
        children(&self.0)
    }

    /// `None` for `with ctx` alone, which covers the rest of the file.
    pub fn body(&self) -> Option<Block> {
        child(&self.0)
    }
}

impl MatchExpr {
    pub fn scrutinee(&self) -> Option<Expr> {
        child(&self.0)
    }

    pub fn arms(&self) -> impl Iterator<Item = MatchArm> + '_ {
        children(&self.0)
    }
}

impl MatchArm {
    pub fn pattern(&self) -> Option<Pattern> {
        child(&self.0)
    }

    /// `g` in `p if g => body`.
    pub fn guard(&self) -> Option<Expr> {
        let arrow = own_token(&self.0, TokenKind::FatArrow).map_or(u32::MAX, |t| t.span().start);
        after::<Expr>(&self.0, TokenKind::If).filter(|g| g.syntax().span().start < arrow)
    }

    /// The body after `=>`, or the reported `p:` arm's block.
    pub fn body(&self) -> Option<ArmBody> {
        after(&self.0, TokenKind::FatArrow).or_else(|| after(&self.0, TokenKind::Colon))
    }
}

impl ParamList {
    /// A lambda's or a table's parameters: patterns, each maybe `mut`
    /// and typed. A fn's are [`ParamList::params`].
    pub fn lambda_params(&self) -> Vec<LambdaParam> {
        let mut out: Vec<LambdaParam> = Vec::new();
        let mut is_mut = false;
        for e in self.0.children_with_tokens() {
            match e {
                SyntaxElement::Token(t) if t.kind() == TokenKind::Mut => is_mut = true,
                SyntaxElement::Token(_) => {}
                SyntaxElement::Node(n) => {
                    if let Some(pattern) = Pattern::cast(n.clone()) {
                        let ty = None;
                        out.push(LambdaParam {
                            is_mut,
                            pattern,
                            ty,
                        });
                        is_mut = false;
                    } else if let (Some(ty), Some(last)) = (Type::cast(n), out.last_mut()) {
                        last.ty = Some(ty);
                    }
                }
            }
        }
        out
    }
}

impl LambdaExpr {
    pub fn params(&self) -> Option<ParamList> {
        child(&self.0)
    }

    /// The return type after `->`.
    pub fn ret(&self) -> Option<Type> {
        after(&self.0, TokenKind::Arrow)
    }

    pub fn body(&self) -> Option<ArmBody> {
        after(&self.0, TokenKind::FatArrow)
    }
}

impl TableExpr {
    pub fn dims(&self) -> Option<Dims> {
        child(&self.0)
    }

    pub fn params(&self) -> Option<ParamList> {
        child(&self.0)
    }

    pub fn body(&self) -> Option<ArmBody> {
        after(&self.0, TokenKind::FatArrow)
    }
}

impl TableFill {
    /// The `[n]` before the value: a one-element list.
    pub fn list(&self) -> Option<ListExpr> {
        child(&self.0)
    }

    /// The value every cell holds.
    pub fn fill(&self) -> Option<Expr> {
        children::<Expr>(&self.0).nth(1)
    }
}

impl ListExpr {
    pub fn elems(&self) -> impl Iterator<Item = Expr> + '_ {
        children(&self.0)
    }
}

impl TupleExpr {
    pub fn elems(&self) -> impl Iterator<Item = Expr> + '_ {
        children(&self.0)
    }
}

impl MapExpr {
    /// None for `[:]`.
    pub fn entries(&self) -> impl Iterator<Item = MapEntry> + '_ {
        children(&self.0)
    }
}

impl MapEntry {
    pub fn key(&self) -> Option<Expr> {
        before(&self.0, TokenKind::Colon)
    }

    pub fn value(&self) -> Option<Expr> {
        after(&self.0, TokenKind::Colon)
    }
}

macro_rules! comprehension {
    ($($node:ident),*) => {$(
        impl $node {
            /// The value made for each binding.
            pub fn body(&self) -> Option<Expr> {
                child(&self.0)
            }

            pub fn clauses(&self) -> impl Iterator<Item = CompClause> + '_ {
                children(&self.0)
            }
        }
    )*};
}

comprehension!(CompExpr, GeneratorExpr);

impl BlockExpr {
    /// Its statements, the value last.
    pub fn stmts(&self) -> impl Iterator<Item = Stmt> + '_ {
        children(&self.0)
    }

    /// Its statements and local items, as written.
    pub fn entries(&self) -> impl Iterator<Item = BlockEntry> + '_ {
        children(&self.0)
    }

    /// `{ s; }`: the block ends with `;`, so its value is `()`.
    pub fn ends_with_semi(&self) -> bool {
        let last = self.0.children_with_tokens().filter(|e| match e {
            SyntaxElement::Token(t) => !t.kind().is_trivia() && t.kind() != TokenKind::RBrace,
            SyntaxElement::Node(_) => true,
        });
        matches!(last.last(), Some(SyntaxElement::Token(t)) if t.kind() == TokenKind::Semi)
    }
}

impl FieldFn {
    /// `0` in `.0`, `name` in `.name`.
    pub fn field(&self) -> Option<SyntaxToken> {
        own_tokens(&self.0).find(|t| matches!(t.kind(), TokenKind::Ident | TokenKind::Int))
    }
}

impl FieldList {
    pub fn fields(&self) -> impl Iterator<Item = FieldDecl> + '_ {
        children(&self.0)
    }
}

impl MutExpr {
    pub fn inner(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl TryExpr {
    pub fn inner(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl UnwrapExpr {
    pub fn inner(&self) -> Option<Expr> {
        child(&self.0)
    }
}

impl WithExpr {
    pub fn base(&self) -> Option<Expr> {
        child(&self.0)
    }

    /// The fields replaced: `f: v` in `x with f: v`.
    pub fn fields(&self) -> impl Iterator<Item = RecordField> + '_ {
        children(&self.0)
    }
}

impl IndexExpr {
    pub fn base(&self) -> Option<Expr> {
        child(&self.0)
    }

    /// The values inside `[..]`, in order: `i`, `j` in `a[i, j]`.
    pub fn indices(&self) -> impl Iterator<Item = Expr> {
        let list = child::<IndexList>(&self.0);
        let exprs: Vec<Expr> = list.iter().flat_map(|l| children(&l.0)).collect();
        exprs.into_iter()
    }
}

impl IsExpr {
    pub fn value(&self) -> Option<Expr> {
        child(&self.0)
    }

    pub fn pattern(&self) -> Option<Pattern> {
        child(&self.0)
    }
}

impl KeepExpr {
    /// The place kept: `best` in `best max= x`.
    pub fn target(&self) -> Option<Expr> {
        child(&self.0)
    }

    /// `max=` rather than `min=`.
    pub fn is_max(&self) -> bool {
        own_tokens(&self.0).any(|t| is_word(&t, Contextual::Max))
    }

    pub fn value(&self) -> Option<Expr> {
        after(&self.0, TokenKind::Eq)
    }
}

impl CompareExpr {
    /// `a`, `b`, `c` in `a < b <= c`.
    pub fn operands(&self) -> impl Iterator<Item = Expr> + '_ {
        children(&self.0)
    }

    /// The links between the operands, in order: `not in` is one [`CmpOp::NotIn`].
    pub fn ops(&self) -> Vec<CmpOp> {
        let mut out = Vec::new();
        let mut negated = false;
        for t in own_tokens(&self.0) {
            if t.kind() == TokenKind::Not {
                negated = true;
                continue;
            }
            if let Some(ops::Kind::Cmp(op)) = ops::binary(t.kind()).map(|b| b.kind) {
                out.push(match op {
                    CmpOp::In if negated => CmpOp::NotIn,
                    op => op,
                });
            }
            negated = false;
        }
        out
    }
}

impl OpValue {
    /// The operator: `+` in `fold(+, xs)`.
    pub fn op(&self) -> Option<SyntaxToken> {
        own_tokens(&self.0).next()
    }
}

impl TuplePat {
    pub fn patterns(&self) -> impl Iterator<Item = Pattern> + '_ {
        children(&self.0)
    }
}

impl ParenPat {
    pub fn pattern(&self) -> Option<Pattern> {
        child(&self.0)
    }
}

impl OrPat {
    pub fn alternatives(&self) -> impl Iterator<Item = Pattern> + '_ {
        children(&self.0)
    }
}

impl LiteralPat {
    /// The value matched: a `Literal`, or `-` and one as a `PrefixExpr`.
    pub fn value(&self) -> Option<Expr> {
        child(&self.0)
    }
}

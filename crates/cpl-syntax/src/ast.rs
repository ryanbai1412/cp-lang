//! The syntax tree.
//!
//! Every span in a tree is relative to the start of the top-level item it
//! belongs to. An edit above an item moves it without changing its text, so
//! its tree, and everything computed from it, is reused as is.
//!
//! The tree is always complete. Where the source is broken the parser puts
//! `Missing` or `Error` nodes, which the checker gives the error type and
//! never reports on again.

use cpl_diag::Span;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

impl Name {
    /// True for names that are types or domains by the naming rule
    /// (`PascalCase`), which the parser relies on to read `Name<...>`.
    pub fn is_type_like(&self) -> bool {
        self.text.starts_with(|c: char| c.is_ascii_uppercase())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

impl Block {
    /// The block can't reach its end: its last statement is a `return`,
    /// `break`, `continue` or trap (`todo`, `assert false`), or an `if` or
    /// `match` each of whose branches can't.
    pub fn leaves(&self) -> bool {
        let Some(last) = self.stmts.last() else {
            return false;
        };
        match &last.kind {
            StmtKind::Return(_) | StmtKind::Break(_) | StmtKind::Continue(_) => true,
            StmtKind::Assert { cond, .. } => matches!(cond.kind, ExprKind::Bool(false)),
            StmtKind::Unchecked(block) => block.leaves(),
            StmtKind::Expr(e) => e.leaves(),
            _ => false,
        }
    }
}

impl Expr {
    /// The value of a whole-number literal: a literal read, maybe negated or written with an
    /// exponent (`1e9`, R01.4.2). An expression that computes, `1e9 + 7`, is not a literal; its
    /// value is worked out by running it.
    pub fn int_literal(&self) -> Option<i64> {
        match &self.kind {
            ExprKind::Int(i) => i64::try_from(*i).ok(),
            ExprKind::Paren(x) => x.int_literal(),
            ExprKind::Unary {
                op: UnOp::Neg,
                expr,
            } => expr.int_literal()?.checked_neg(),
            ExprKind::Float(text) => {
                let (m, e) = text.split_once(['e', 'E'])?;
                m.parse::<i64>()
                    .ok()?
                    .checked_mul(10i64.checked_pow(e.parse().ok()?)?)
            }
            _ => None,
        }
    }
    /// `base[indices]` over `span`: how a table of tables' `dp[r, c, k]`
    /// splits into the cell `dp[r, c]` and its own index `k`.
    pub fn indexed(base: &Expr, indices: &[Expr], span: Span) -> Expr {
        Expr {
            kind: ExprKind::Index {
                base: Box::new(base.clone()),
                indices: indices.to_vec(),
            },
            span,
        }
    }

    /// A bare generator argument, `max(x for x in xs)`, not a list
    /// comprehension `[x for x in xs]`, which starts at its `[`.
    pub fn is_generator(&self) -> bool {
        matches!(&self.kind, ExprKind::Comprehension { body, .. } if body.span.start == self.span.start)
    }

    /// Running the expression never gives a value; see [`Block::leaves`].
    pub fn leaves(&self) -> bool {
        match &self.kind {
            ExprKind::Name(name) => name == "todo",
            ExprKind::Block(block) => block.leaves(),
            ExprKind::If {
                branches,
                otherwise: Some(otherwise),
            } => otherwise.leaves() && branches.iter().all(|(_, b)| b.leaves()),
            ExprKind::Match { arms, .. } => {
                !arms.is_empty() && arms.iter().all(|a| a.body.leaves())
            }
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Decorator {
    /// `@!name`: a file-level directive, in force from its line down. A
    /// plain `@name` decorates the one item below it.
    pub directive: bool,
    /// What `name` spells, read once by the parser: `None` for a name outside the closed set.
    pub mark: Option<crate::decorators::Mark>,
    pub name: Name,
    pub args: Vec<Arg>,
    pub span: Span,
}

#[derive(PartialEq, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
    pub decorators: Vec<Decorator>,
    /// Declared with `pub`: visible to modules that `use` this one.
    pub public: bool,
}

impl Stmt {
    /// Its decorator `m`, if it has one.
    pub fn mark(&self, m: crate::decorators::Mark) -> Option<&Decorator> {
        self.decorators.iter().find(|d| d.mark == Some(m))
    }
}

impl Stmt {
    /// The right-hand side of `let`, an assignment or `return` when a `?` in
    /// it has the whole side as its extent: that side is the optional, never
    /// a way out of the fn. (R02.4.7)
    pub fn rhs_extent(&self) -> Option<Span> {
        let rhs = match &self.kind {
            StmtKind::Let {
                init: Some(e),
                otherwise: None,
                ..
            }
            | StmtKind::Assign { value: e, .. }
            | StmtKind::Return(Some(e)) => e,
            _ => return None,
        };
        let mut e = rhs;
        while let ExprKind::Paren(inner) = &e.kind {
            e = inner;
        }
        rhs.bare_try().map(|_| e.span)
    }

    /// The table a `@memo(into: dp)` fn keeps its answers in: its name and
    /// where the name is written.
    pub fn memo_into(&self) -> Option<(&str, Span)> {
        let memo = self.decorators.iter().find(|d| d.name.text == "memo")?;
        memo.args
            .iter()
            .find_map(|arg| match (&arg.name, &arg.value.kind) {
                (Some(label), ExprKind::Name(table)) if label.text == "into" => {
                    Some((table.as_str(), arg.value.span))
                }
                _ => None,
            })
    }
}

/// `T: Ord + Bounded`, from a type parameter list or a `where` clause.
#[derive(Clone, PartialEq, Debug)]
pub struct Bound {
    pub name: Name,
    pub traits: Vec<Type>,
}

/// `lends T`, `lends mut T`, or `lends(a) T` in a projection's signature.
#[derive(Clone, PartialEq, Debug)]
pub struct Lends {
    pub sources: Vec<Name>,
    pub mutable: bool,
    pub ty: Type,
    /// The `lends` keyword, when written without the `->` before it.
    pub bare: Option<Span>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StmtKind {
    /// `let pat: ty = init` or `var ...`.
    Let {
        mutable: bool,
        pat: Pattern,
        ty: Option<Type>,
        init: Option<Expr>,
        /// `let v = u else: return`: runs when the value is `none`.
        otherwise: Option<Block>,
        /// `let k: 0..n = read() where k < n`: a refinement of the input.
        guard: Option<Expr>,
    },
    Const {
        name: Name,
        /// `const Sum<T> = ... where T: Add`: one value per choice of `T`.
        generics: Vec<Name>,
        bounds: Vec<Bound>,
        ty: Option<Type>,
        /// Absent only in a trait, where the implementor supplies it.
        init: Option<Expr>,
    },
    Fn(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Domain {
        name: Name,
        def: Expr,
        /// Written as a count alone, `[m]`, which `def` holds as `0..m` (R10.2.2).
        counted: bool,
    },
    TypeAlias {
        name: Name,
        generics: Vec<Name>,
        ty: Type,
        /// `type Arc: Target<V>` in a trait: the traits the implementor's type must have.
        bounds: Vec<Type>,
    },
    Trait {
        name: Name,
        generics: Vec<Name>,
        /// `trait Add<Rhs = Self>`: what a parameter is when a use leaves it out.
        generic_defaults: Vec<Option<Type>>,
        /// `trait Seq: Index`.
        supers: Vec<Type>,
        body: Block,
        /// `trait Both = Sh + Nm`: a name for its bounds, never implemented.
        alias: bool,
        /// `where Self.Arc: Weighted<W>` on an alias: bounds on the traits' associated types.
        bounds: Vec<Bound>,
    },
    /// `mod name:` — an inline module whose `pub` items are accessed as `name.item`.
    Mod {
        name: Name,
        body: Block,
    },
    Impl {
        trait_name: Option<Type>,
        target: Type,
        bounds: Vec<Bound>,
        body: Block,
    },
    /// `law assoc(a: T, b: T) = op(a, b) == op(b, a)` in a struct, or
    /// `law disjoint(i, j) = i != j` in a projection.
    Law {
        name: Name,
        params: Vec<Param>,
        body: Expr,
    },
    /// `lend place` or `lend mut place`, the last statement of a projection.
    Lend {
        mutable: bool,
        place: Expr,
    },
    Use {
        path: Vec<Name>,
        /// `use a.b.{X, Y}`.
        names: Vec<Name>,
        glob: bool,
        alias: Option<Name>,
    },
    Expr(Expr),
    Assign {
        target: Expr,
        /// `=`, `+=`, `max=`, ...
        op: AssignOp,
        value: Expr,
    },
    While {
        cond: Cond,
        body: Block,
        else_body: Option<Block>,
    },
    For {
        label: Option<Name>,
        pat: Pattern,
        iter: Expr,
        body: Block,
        else_body: Option<Block>,
    },
    Return(Option<Expr>),
    Break(Option<Name>),
    Continue(Option<Name>),
    /// `yield x`, or `yield from xs` to yield each value of `xs`.
    Yield {
        value: Expr,
        from: bool,
    },
    Assert {
        cond: Expr,
        message: Option<Expr>,
    },
    Unchecked(Block),
    /// `with io(sep: ",")`: a context for `body`, or for the rest of the
    /// enclosing block when written bare. (R09.1.1)
    With {
        ctx: Name,
        args: Vec<Arg>,
        body: Option<Block>,
    },
    /// `context opt:\n    k: Int = 2`: user-defined context declaration.
    Context {
        name: Name,
        fields: Vec<FieldDecl>,
    },
    /// A statement the parser could not read. Already reported.
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AssignOp {
    Set,
    Binary(BinOp),
    Max,
    Min,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FnDecl {
    pub name: Name,
    pub generics: Vec<Name>,
    pub bounds: Vec<Bound>,
    pub params: Vec<Param>,
    pub ret: Option<Type>,
    /// `fn at(self, i: V) lends mut Int`: a projection.
    pub lends: Option<Lends>,
    /// `fn pick() -> Int uses opt`: contexts used by this fn.
    pub uses: Vec<Name>,
    pub body: FnBody,
    /// From `fn` to the end of the signature.
    pub sig_span: Span,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FnBody {
    Block(Block),
    /// `fn sq(x: Int) -> Int = x * x`.
    Expr(Expr),
    /// A declaration without a body, as in a trait.
    None,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Param {
    pub mutable: bool,
    pub variadic: bool,
    /// `lazy f: () -> T`: a caller passes an expression of type `T`, and it
    /// becomes `() => expr`.
    pub lazy: bool,
    /// After `;` in the parameter list: can only be passed by name.
    pub named_only: bool,
    /// Optional external call-site label (`while ok: (K) -> Bool`), so any
    /// function can declare a keyword-labelled parameter without shadowing.
    pub label: Option<Name>,
    pub name: Name,
    pub ty: Option<Type>,
    pub default: Option<Expr>,
    pub span: Span,
}

impl Param {
    /// Name callers write when passing this parameter by name.
    pub fn label_text(&self) -> &str {
        self.label
            .as_ref()
            .map_or(self.name.text.as_str(), |l| l.text.as_str())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct StructDecl {
    pub name: Name,
    pub generics: Vec<Name>,
    pub generic_defaults: Vec<Option<Type>>,
    pub bounds: Vec<Bound>,
    /// Header fields (`struct Monoid<T>(id: T, op: (T, T) -> T)`) first,
    /// then the fields of the body.
    pub fields: Vec<FieldDecl>,
    /// Everything in the body that is not a field: fns, laws, owned
    /// domains, constants.
    pub methods: Vec<Stmt>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FieldDecl {
    pub name: Name,
    pub ty: Type,
    pub default: Option<Expr>,
    /// `var top = 0`: assignable after construction.
    pub mutable: bool,
    /// `@cache parent: ...`: interior-mutable cache field, skipped by `==`.
    pub cached: bool,
    /// `@bits w: List<U64>`: one bit per position of the struct's domain
    /// parameter, 64 to a word, with the words past the domain zero.
    pub bits: bool,
    /// `pub x: Int`: visible outside the declaring module (R11.1.4).
    pub public: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct EnumDecl {
    pub name: Name,
    pub generics: Vec<Name>,
    pub variants: Vec<Variant>,
    pub methods: Vec<Stmt>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Variant {
    pub name: Name,
    pub fields: Vec<(Option<Name>, Type)>,
    pub value: Option<Expr>,
    pub tag: Option<Expr>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Arg {
    pub name: Option<Name>,
    pub mutable: bool,
    pub value: Expr,
}

/// The condition of an `if` or `while`.
#[derive(Clone, PartialEq, Debug)]
pub enum Cond {
    Expr(Expr),
    /// `if let v = find():`, `if let b = before[c] and b < d:`,
    /// `if let a = i - 1 and let b = j - 1:`.
    Let {
        pat: Pattern,
        value: Expr,
        /// What must also hold, with the pattern's names in scope: a
        /// condition, or another `let`.
        and: Option<Box<Cond>>,
    },
    /// `if a == 1, let b = p:`: a test, then after the `,` what must
    /// also hold.
    Guard {
        test: Expr,
        and: Box<Cond>,
    },
    /// `if const D.max_len <= 64:`.
    Const(Expr),
}

#[derive(PartialEq, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    /// The first `?` whose extent is this expression's: operators, calls,
    /// fields and indexing reach it; `??` and lambdas are extents of their
    /// own. (R02.4.7)
    pub fn bare_try(&self) -> Option<Span> {
        match &self.kind {
            ExprKind::Try(_) => Some(self.span),
            ExprKind::Binary {
                op: BinOp::Coalesce,
                ..
            } => None,
            ExprKind::Binary { lhs, rhs, .. } => lhs.bare_try().or_else(|| rhs.bare_try()),
            ExprKind::Compare { first, rest } => first
                .bare_try()
                .or_else(|| rest.iter().find_map(|(_, rhs)| rhs.bare_try())),
            ExprKind::Unary { expr: operand, .. }
            | ExprKind::Paren(operand)
            | ExprKind::Unwrap(operand)
            | ExprKind::Field { base: operand, .. } => operand.bare_try(),
            ExprKind::Call { callee, args } => callee
                .bare_try()
                .or_else(|| args.iter().find_map(|a| a.value.bare_try())),
            ExprKind::Index { base, indices } => base
                .bare_try()
                .or_else(|| indices.iter().find_map(Expr::bare_try)),
            _ => None,
        }
    }

    /// `[x for x in base]`: the walk of the domain `base` names, as a list.
    pub fn walk_list(base: &Expr) -> Expr {
        let span = base.span;
        let item = "walked".to_string();
        let body = Expr {
            kind: ExprKind::Name(item.clone()),
            span,
        };
        let clause = Clause {
            pat: Pattern {
                kind: PatternKind::Name(item),
                span,
            },
            iter: base.clone(),
            conds: Vec::new(),
        };
        Expr {
            kind: ExprKind::Comprehension {
                body: Box::new(body),
                clauses: vec![clause],
            },
            span,
        }
    }

    pub fn variant_list(base: &Expr, variants: &[String]) -> Expr {
        let span = base.span;
        let item = |v: &String| Expr {
            kind: ExprKind::Field {
                base: Box::new(base.clone()),
                name: Name {
                    text: v.clone(),
                    span,
                },
            },
            span,
        };
        Expr {
            kind: ExprKind::List(variants.iter().map(item).collect()),
            span,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ExprKind {
    Int(i128),
    Float(String),
    Str(Vec<StrPiece>),
    Char(char),
    Bool(bool),
    None,
    Name(String),
    SelfValue,
    /// `@cpp("...")`: a fn's body written in C++, which the bundle emits as
    /// written (Rbundle.5).
    Cpp(String),
    /// `@intrinsic("id")`: a fn's body that is one primitive of the
    /// intrinsics table, run by both engines (R12.14.1).
    Intrinsic(String),
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinOp,
        op_span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `lo <= x < hi`: operands and the operators between them.
    Compare {
        first: Box<Expr>,
        rest: Vec<(CmpOp, Expr)>,
    },
    Range {
        lo: Option<Box<Expr>>,
        hi: Option<Box<Expr>>,
        inclusive: bool,
        step: Option<Box<Expr>>,
    },
    /// `(lo..hi).rev by step`: a walk taken every `step`-th value, from
    /// where it starts.
    By {
        walk: Box<Expr>,
        step: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Arg>,
    },
    Index {
        base: Box<Expr>,
        indices: Vec<Expr>,
    },
    /// `base.name`. The name is empty while the user is still typing it.
    Field {
        base: Box<Expr>,
        name: Name,
    },
    /// `Name<T, U>` or `name::<T>` in an expression.
    Generic {
        base: Box<Expr>,
        args: Vec<Type>,
    },
    /// `best min= d` where a value is wanted: a `Bool`, true when the
    /// place changed. As a statement it is a [`StmtKind::Assign`].
    Keep {
        target: Box<Expr>,
        max: bool,
        value: Box<Expr>,
    },
    Paren(Box<Expr>),
    Tuple(Vec<Expr>),
    Record(Vec<(Name, Expr)>),
    List(Vec<Expr>),
    /// `[3: 4, 5: 6]`.
    Map(Vec<(Expr, Expr)>),
    /// `[body for pat in iter if cond]`, or the same without brackets as
    /// the only argument of a call: `max(dist[v] for v in V)`.
    Comprehension {
        body: Box<Expr>,
        /// One per `for`, outermost first.
        clauses: Vec<Clause>,
    },
    /// `[V] u => deg[u] * 2`.
    Table {
        domains: Vec<Type>,
        params: Vec<Pattern>,
        body: Box<Expr>,
    },
    /// `[w.keys]0` or `[0..n]0`: table over a key space filled with a constant value.
    TableFill {
        domain: Box<Expr>,
        value: Box<Expr>,
    },
    Lambda {
        params: Vec<Pattern>,
        param_tys: Vec<Option<Type>>,
        /// `(mut d, e) => ..`: empty unless some parameter is `mut`.
        param_mut: Vec<bool>,
        ret_ty: Option<Type>,
        body: Box<Expr>,
    },
    /// `{ let k = key(a); (k.x, -k.y) }`.
    Block(Block),
    If {
        branches: Vec<(Cond, Block)>,
        otherwise: Option<Block>,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<Arm>,
    },
    /// `e with w: 7`.
    With {
        base: Box<Expr>,
        updates: Vec<(Name, Expr)>,
    },
    /// `(to: r.v, ..e without (u, v))` or `(..e without (u, v))`.
    Without {
        prefix: Vec<(Name, Expr)>,
        base: Box<Expr>,
        removed: Vec<Name>,
    },
    /// `if T: Trait:` or `if M: Selective<T>:`, a compile-time trait bound test.
    HasBound {
        param: Name,
        /// The trait with its args, a `TypeKind::Named`.
        bound: Type,
    },
    /// Postfix `?`.
    Try(Box<Expr>),
    /// Postfix `!`: the value of an optional that is known to be there.
    Unwrap(Box<Expr>),
    /// `mut place` in `let st = mut a[i]`: a named mutable alias to a place.
    Mut(Box<Expr>),
    /// An expression was expected and nothing was there. Already reported.
    Missing,
    /// Tokens the parser could not read as an expression. Already reported.
    Error,
}

/// `for pat in iter [if cond]` in a comprehension.
#[derive(Clone, PartialEq, Debug)]
pub struct Clause {
    pub pat: Pattern,
    pub iter: Expr,
    /// Each `if cond` or `if let pat = value` after the iterator.
    pub conds: Vec<Cond>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StrPiece {
    Text(String),
    Expr(Expr),
    /// `{x:.2f}`, `{n:05}`, `{n:x}`: a number in the format after the `:`.
    Spec(Expr, FmtSpec),
}

pub use crate::literal::{FmtKind, FmtSpec};

#[derive(Clone, PartialEq, Debug)]
pub struct Arm {
    pub pat: Pattern,
    pub guard: Option<Expr>,
    pub body: Block,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UnOp {
    Pos,
    Neg,
    Not,
    BitNot,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    And,
    Or,
    Coalesce,
}

impl Expr {
    /// The fn item a value names, as `type_of(..)` keys it: a name (`max`,
    /// `gcd`) or an operator written bare (`+`).
    pub fn fn_item_key(&self) -> Option<String> {
        match &self.kind {
            ExprKind::Paren(e) => e.fn_item_key(),
            ExprKind::Name(n) => Some(n.to_string()),
            ExprKind::Lambda { body, .. } => match &body.kind {
                ExprKind::Binary { op, op_span, .. } if *op_span == self.span => {
                    Some(op.text().to_string())
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// An identifier for the fn item a value names: `Max` for `max`, `Add`
    /// for `+`.
    pub fn fn_item_ident(&self) -> Option<String> {
        match &self.kind {
            ExprKind::Paren(e) => e.fn_item_ident(),
            ExprKind::Name(n) => Some(n.to_string()),
            ExprKind::Lambda { body, .. } => match &body.kind {
                ExprKind::Binary { op, op_span, .. } if *op_span == self.span => {
                    Some(format!("{op:?}"))
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// The type `type_of(..)` of the fn item this value names is filed under.
    pub fn fn_item_type(&self) -> Option<String> {
        self.fn_item_key().map(|k| format!("type_of({k})"))
    }
}

impl BinOp {
    pub fn text(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Pow => "**",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::And => "and",
            BinOp::Or => "or",
            BinOp::Coalesce => "??",
        }
    }
}

/// The trait and method that give an operator its meaning on a type that declares them
/// (R08.1.5): the one table both the parser's `fn (a) < (b)` and the checker read.
impl CmpOp {
    pub fn text(self) -> &'static str {
        match self {
            CmpOp::Eq => "==",
            CmpOp::Ne => "!=",
            CmpOp::Lt => "<",
            CmpOp::Le => "<=",
            CmpOp::Gt => ">",
            CmpOp::Ge => ">=",
            CmpOp::In => "in",
            CmpOp::NotIn => "not in",
        }
    }
    pub fn trait_method(self) -> Option<(&'static str, &'static str)> {
        match self {
            CmpOp::Eq | CmpOp::Ne => Some(("Eq", "eq")),
            CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge => Some(("Ord", "lt")),
            CmpOp::In | CmpOp::NotIn => None,
        }
    }
}

impl BinOp {
    pub fn trait_method(self) -> Option<(&'static str, &'static str)> {
        Some(match self {
            BinOp::Add => ("Add", "add"),
            BinOp::Sub => ("Sub", "sub"),
            BinOp::Mul => ("Mul", "mul"),
            BinOp::Div => ("Div", "div"),
            BinOp::Rem => ("Rem", "rem"),
            BinOp::Pow => ("Pow", "pow"),
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    NotIn,
}

#[derive(PartialEq, Debug)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

impl Pattern {
    /// A capitalized name alone, `L` or `MOD`, which parses as a variant
    /// with no arguments: the name.
    /// The one name a `let` of this pattern binds: `x`, a domain `D`, or a capitalised `K`.
    pub fn let_name(&self) -> Option<&str> {
        match &self.kind {
            PatternKind::Name(n) => Some(n),
            PatternKind::Domain(n) => Some(&n.text),
            _ => self.bare_variant(),
        }
    }

    pub fn bare_variant(&self) -> Option<&str> {
        match &self.kind {
            PatternKind::Variant { path, args } if path.len() == 1 && args.is_empty() => {
                Some(&path[0].text)
            }
            _ => None,
        }
    }

    /// The patterns directly inside a tuple, variant or record pattern.
    pub fn parts(&self) -> Box<dyn Iterator<Item = &Pattern> + '_> {
        match &self.kind {
            PatternKind::Tuple(items) | PatternKind::Variant { args: items, .. } => {
                Box::new(items.iter())
            }
            PatternKind::Record { fields, .. } => Box::new(fields.iter().map(|(_, p)| p)),
            _ => Box::new(std::iter::empty()),
        }
    }

    /// A record pattern as the tuple pattern over a record with fields
    /// `names`, in their order, `_` where `..` skips one. `Err` names a
    /// field the record lacks, or one the pattern leaves out without `..`.
    pub fn record_as_tuple(&self, names: &[&str]) -> Result<Pattern, cpl_diag::msgs::Said> {
        let PatternKind::Record { fields, rest } = &self.kind else {
            return Ok(self.clone());
        };
        if let Some((name, _)) = fields
            .iter()
            .find(|(n, _)| !names.contains(&n.text.as_str()))
        {
            return Err(cpl_diag::msgs::syntax::RECORD_HAS_NO_FIELD.at(&[&name.text]));
        }
        let mut items = Vec::with_capacity(names.len());
        for &name in names {
            match fields.iter().find(|(n, _)| n.text == name) {
                Some((_, p)) => items.push(p.clone()),
                None if *rest => items.push(Pattern {
                    kind: PatternKind::Wildcard,
                    span: self.span,
                }),
                None => return Err(cpl_diag::msgs::syntax::PATTERN_LEAVES_OUT.at(&[&name])),
            }
        }
        Ok(Pattern {
            kind: PatternKind::Tuple(items),
            span: self.span,
        })
    }

    /// The one parameter of a lambda given where a one-argument fn is
    /// wanted: `(a, b) => ..` takes a pair apart (R06.1.4).
    pub fn single_param(params: &[Pattern]) -> Option<Pattern> {
        match params {
            [] => None,
            [one] => Some(one.clone()),
            [first, .., last] => Some(Pattern {
                kind: PatternKind::Tuple(params.to_vec()),
                span: Span::new(first.span.start, last.span.end),
            }),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum PatternKind {
    Wildcard,
    Name(String),
    /// `domain P` in a binding pattern: `let (domain P, part) = f(..)`.
    Domain(Name),
    Tuple(Vec<Pattern>),
    /// `(to: u, w: c)`, `(total:, take:)`, `(to: u, ..)`: a record's fields
    /// by name; `rest` is the `..` that skips the others (R02.7.6).
    Record {
        fields: Vec<(Name, Pattern)>,
        rest: bool,
    },
    /// `Add(l, r)` or a bare variant such as `L`.
    Variant {
        path: Vec<Name>,
        args: Vec<Pattern>,
    },
    Literal(Box<Expr>),
    Or(Vec<Pattern>),
    /// `var x` inside a destructuring pattern: `for (var left, v) in ...:`.
    Var(Box<Pattern>),
    Error,
}

#[derive(PartialEq, Debug)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

impl Type {
    /// The type a bare name spells, with no type arguments.
    pub fn named(text: &str, span: Span) -> Type {
        Type {
            kind: TypeKind::Named {
                name: Name {
                    text: text.to_string(),
                    span,
                },
                args: Vec::new(),
            },
            span,
        }
    }

    /// The index type an expression in `[e].filled(x)` stands for, read as
    /// `[e]T` reads it: a capitalized name is a type (`[V]`), a field off one
    /// is a value of it (`[V.cuts]`), anything else is counted, `0..e`
    /// (`[n]`, `[w.keys]`).
    pub fn of_index(e: &Expr) -> Type {
        let capital = |n: &str| n.starts_with(|c: char| c.is_ascii_uppercase());
        let kind = match &e.kind {
            ExprKind::Name(n) if capital(n) => TypeKind::Named {
                name: Name {
                    text: n.clone(),
                    span: e.span,
                },
                args: Vec::new(),
            },
            ExprKind::Field { base, .. } if matches!(&base.kind, ExprKind::Name(n) if capital(n)) => {
                TypeKind::Value(Box::new(e.clone()))
            }
            _ => TypeKind::Range {
                lo: Some(Box::new(Expr {
                    kind: ExprKind::Int(0),
                    span: Span::point(e.span.start),
                })),
                hi: Some(Box::new(e.clone())),
                inclusive: false,
            },
        };
        Type { kind, span: e.span }
    }

    /// `gen<T>(args)` for a named `T`: the call `T.gen(args)`, which a
    /// `Random` impl gives (R06.4.3).
    pub fn gen_call(&self, args: &[Arg], span: Span) -> Option<Expr> {
        let TypeKind::Named { name, args: tys } = &self.kind else {
            return None;
        };
        let head = Expr {
            kind: ExprKind::Name(name.text.clone()),
            span: name.span,
        };
        let base = if tys.is_empty() {
            head
        } else {
            Expr {
                kind: ExprKind::Generic {
                    base: Box::new(head),
                    args: tys.clone(),
                },
                span: self.span,
            }
        };
        let callee = Expr {
            kind: ExprKind::Field {
                base: Box::new(base),
                name: Name {
                    text: "gen".into(),
                    span: self.span,
                },
            },
            span,
        };
        Some(Expr {
            kind: ExprKind::Call {
                callee: Box::new(callee),
                args: args.to_vec(),
            },
            span,
        })
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TypeKind {
    /// `Int`, `List<Int>`, `V`.
    Named {
        name: Name,
        args: Vec<Type>,
    },
    /// `[V]Int`, `[Row, Col]Char`.
    Array {
        index: Vec<Type>,
        elem: Box<Type>,
    },
    /// `[domain S]Char`: an array that brings its own domain.
    OwnDomain {
        name: Name,
        elem: Box<Type>,
    },
    Optional(Box<Type>),
    Tuple(Vec<Type>),
    Record(Vec<(Name, Type)>),
    /// `0..=1e9`, `1..=n`, `0..`, `..=8`.
    Range {
        lo: Option<Box<Expr>>,
        hi: Option<Box<Expr>>,
        inclusive: bool,
    },
    Fn {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    /// `mut T`: a mutable parameter in a fn type `(mut S, T) -> ()`.
    Mut(Box<Type>),
    /// `yields T`: the return type of a generator.
    Yields(Box<Type>),
    /// A value in a type argument list: the `7` of `Mod<7>`.
    Value(Box<Expr>),
    /// `domain C` in a returned record, `-> (domain C, comp: [V]C)`: the
    /// fn makes a fresh domain and names it for the caller.
    NewDomain,
    /// `domain P` in a returned tuple, `-> (domain P, [P]Int)`: an
    /// existential fresh domain bound for later elements of the tuple.
    DomainBinder(Name),
    /// `(lo: Int, hi: Int, ..)`: open row bound on a record type parameter.
    Row(Vec<(Name, Type)>),
    /// `(to: V, ..R without (u, v))` or `(..R without (u, v))`: record type with named fields added and/or removed.
    RowWithout {
        prefix: Vec<(Name, Type)>,
        base: Box<Type>,
        removed: Vec<Name>,
    },
    /// `Str of "01"`, `Str(len: 1..=n)`, or both: a string whose characters
    /// belong to the alphabet and whose length is in the range.
    ConstrainedStr {
        alphabet: Option<String>,
        len: Option<Box<Type>>,
    },
    /// `'+' | '-'`: finite union of literal values.
    LiteralUnion(Vec<String>),
    /// `type_of(max)`, `type_of(+)`: the zero-size type of one fn item, an
    /// impl target apart from its fn type (R08.1.9).
    ItemOf(Box<Expr>),
    /// A field written `total = 0`: its type is its default's.
    Infer,
    Error,
}

impl TypeKind {
    /// The alphabet of `Str of "…"`, when this is one.
    pub fn alphabet(&self) -> Option<&str> {
        match self {
            TypeKind::ConstrainedStr { alphabet, .. } => alphabet.as_deref(),
            _ => None,
        }
    }
}

// The AST's big types clone through one out-of-line fn each: a derived
// `Clone` is `#[inline]`, so every crate that clones one kept its own copy
// of the whole recursive clone (cpl.wasm carried five of `Expr`'s).
impl Clone for Stmt {
    fn clone(&self) -> Self {
        Stmt {
            kind: self.kind.clone(),
            span: self.span,
            decorators: self.decorators.clone(),
            public: self.public,
        }
    }
}

impl Clone for Expr {
    fn clone(&self) -> Self {
        Expr {
            kind: self.kind.clone(),
            span: self.span,
        }
    }
}

impl Clone for Pattern {
    fn clone(&self) -> Self {
        Pattern {
            kind: self.kind.clone(),
            span: self.span,
        }
    }
}

impl Clone for Type {
    fn clone(&self) -> Self {
        Type {
            kind: self.kind.clone(),
            span: self.span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memo_into_names_the_table_and_bare_variant_a_capitalized_name() {
        let s: Vec<Stmt> = crate::parse_file(
            "@memo(into: dp)\nfn f(i: Int) -> Int:\n    return i\nlet MOD = 5\nlet m = 5\n",
        )
        .items
        .into_iter()
        .flat_map(|(_, item)| item.stmts)
        .collect();
        assert_eq!(s[0].memo_into().map(|(t, _)| t), Some("dp"));
        assert_eq!(s[1].memo_into(), None);
        let pat = |s: &Stmt| match &s.kind {
            StmtKind::Let { pat, .. } => pat.bare_variant().map(str::to_string),
            _ => panic!("a let"),
        };
        assert_eq!(pat(&s[1]), Some("MOD".to_string()));
        assert_eq!(pat(&s[2]), None);
    }
}

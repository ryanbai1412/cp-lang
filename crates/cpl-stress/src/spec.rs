//! What a program reads, from its own declarations.
//!
//! ```text
//! let (n, q): (1..=2e5, 1..=2e5) = read()
//! domain V = 1..=n
//! let a: [V]0..=1e9 = read()
//! let queries: [0..q](V, V) = read()
//! ```
//!
//! is an input of two numbers, then `n` numbers, then `q` pairs of
//! vertices. Bounds can use the values read before them, which is what
//! lets the generator and the shrinker keep every case valid.

use cpl_diag::Span;
use cpl_syntax::ast::{Block, FnBody, Stmt,
    BinOp, Expr, ExprKind, Pattern, PatternKind, StmtKind, Type, TypeKind, UnOp,
};
use std::collections::{HashMap, HashSet};

/// The values `Int` takes when a program gives it no range.
pub const DEFAULT_INT: (i128, i128) = (-1_000_000_000, 1_000_000_000);

/// The shape of one value of the input.
#[derive(Clone, PartialEq, Debug)]
pub enum Shape {
    /// An integer in `lo..=hi`; the bounds are evaluated when a case is
    /// made, against the values read so far.
    Int {
        lo: Expr,
        hi: Expr,
    },
    /// `len` values of the same shape.
    Array {
        len: Expr,
        elem: Box<Shape>,
    },
    /// A tuple, or a record `(u: V, v: V)`: its fields in order.
    Tuple(Vec<Shape>),
    /// One byte in `lo..=hi`: a lowercase letter unless told otherwise.
    Char {
        lo: u8,
        hi: u8,
    },
    /// `0` or `1`.
    Bool,
    /// A word of letters, of any length from 1.
    Str,
    /// Every value of `lo..lo + len` once, in some order.
    Perm {
        lo: Expr,
        len: Expr,
    },
    /// The `hi - lo` edges of a tree on the vertices `lo..=hi`, each a
    /// pair of them.
    Tree {
        lo: Expr,
        hi: Expr,
    },
}

/// One `let ... = read()`.
#[derive(Clone, PartialEq, Debug)]
pub struct Field {
    /// The names it binds, in the order their values are read. A name is
    /// `None` where the pattern binds something that is not one integer.
    pub names: Vec<Option<String>>,
    pub shape: Shape,
    /// 1-based.
    pub line: u32,
}

/// Everything a program reads, in order.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct InputSpec {
    pub fields: Vec<Field>,
    /// Constants the bounds use: `const K = 20`.
    pub consts: HashMap<String, i128>,
    /// Names that decide how much input there is (`n` in `1..=n`). The
    /// generator keeps these small and the shrinker lowers them first.
    pub sizes: HashSet<String>,
    /// `@!multitest`: what one test case reads, below the divider. The
    /// input is `fields`, then the number of test cases, then each one.
    pub cases: Option<Box<InputSpec>>,
    /// `@!multitest(until_eof)`: the test cases come with no count.
    pub until_eof: bool,
}

/// A `read()` whose type the generator cannot produce.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SpecError {
    /// In the file.
    pub span: Span,
    pub message: String,
}

fn int(value: i128) -> Expr {
    Expr {
        kind: ExprKind::Int(value),
        span: Span::new(0, 0),
    }
}

fn binary(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr {
        span: lhs.span,
        kind: ExprKind::Binary {
            op,
            op_span: lhs.span,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
    }
}

/// The values of the names read so far.
pub type Env = HashMap<String, i128>;

/// The value of a bound, when it only uses numbers, names in `env`, and
/// `+ - * /`, `min` and `max`.
pub fn eval(expr: &Expr, env: &Env) -> Option<i128> {
    match &expr.kind {
        ExprKind::Int(value) => Some(*value),
        ExprKind::Name(name) => env.get(name).copied(),
        ExprKind::Paren(inner) => eval(inner, env),
        ExprKind::Unary {
            op: UnOp::Neg,
            expr,
        } => eval(expr, env)?.checked_neg(),
        ExprKind::Binary { op, lhs, rhs, .. } => {
            let (a, b) = (eval(lhs, env)?, eval(rhs, env)?);
            match op {
                BinOp::Add => a.checked_add(b),
                BinOp::Sub => a.checked_sub(b),
                BinOp::Mul => a.checked_mul(b),
                BinOp::Div if b != 0 => Some(a.div_euclid(b)),
                _ => None,
            }
        }
        ExprKind::Call { callee, args } => {
            let ExprKind::Name(name) = &callee.kind else {
                return None;
            };
            let values: Option<Vec<i128>> = args.iter().map(|arg| eval(&arg.value, env)).collect();
            match name.as_str() {
                "min" => values?.into_iter().min(),
                "max" => values?.into_iter().max(),
                _ => None,
            }
        }
        _ => None,
    }
}

fn names_in(expr: &Expr, out: &mut HashSet<String>) {
    match &expr.kind {
        ExprKind::Name(name) => {
            out.insert(name.clone());
        }
        ExprKind::Paren(inner) | ExprKind::Unary { expr: inner, .. } => names_in(inner, out),
        ExprKind::Binary { lhs, rhs, .. } => {
            names_in(lhs, out);
            names_in(rhs, out);
        }
        ExprKind::Call { args, .. } => args.iter().for_each(|arg| names_in(&arg.value, out)),
        _ => {}
    }
}

/// The names a program sizes something by, wherever it does: the index
/// of an array (`var dp: [0..n]Int = 0`), a domain (`domain V = 1..=n`),
/// a counted loop (`for _ in 0..q`), `List.filled(n, 0)`.
struct Sizes<'a>(&'a mut HashSet<String>);

impl Sizes<'_> {
    fn of_type(&mut self, ty: &Type) {
        match &ty.kind {
            TypeKind::Array { index, elem } => {
                for dimension in index {
                    match &dimension.kind {
                        TypeKind::Range { lo, hi, .. } => {
                            names_in(lo, self.0);
                            names_in(hi, self.0);
                        }
                        _ => self.of_type(dimension),
                    }
                }
                self.of_type(elem);
            }
            TypeKind::Named { args, .. } | TypeKind::Tuple(args) => args.iter().for_each(|arg| self.of_type(arg)),
            TypeKind::Record(fields) => fields.iter().for_each(|(_, field)| self.of_type(field)),
            _ => {}
        }
    }

    fn of_range(&mut self, range: &Expr) {
        match &range.kind {
            ExprKind::Paren(inner) => self.of_range(inner),
            ExprKind::Range { lo, hi, .. } => {
                for end in [lo, hi].into_iter().flatten() {
                    names_in(end, self.0);
                }
            }
            _ => {}
        }
    }
}

impl cpl_syntax::visit::Visitor for Sizes<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let { ty: Some(ty), .. } => self.of_type(ty),
            StmtKind::Domain { def, .. } => self.of_range(def),
            StmtKind::For { iter, .. } => self.of_range(iter),
            _ => {}
        }
    }

    fn expr(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return;
        };
        let filled = matches!(&callee.kind, ExprKind::Field { base, name }
            if name.text == "filled" && matches!(&base.kind, ExprKind::Name(list) if list == "List"));
        if let (true, Some(len)) = (filled, args.first()) {
            names_in(&len.value, self.0);
        }
    }
}

/// A plain `Int` that is read and then used as a size is at least 1: a
/// problem's `n` counts something, and a negative count stops every
/// program on its first array.
fn sizes_count(spec: &mut InputSpec) {
    let plain = Shape::Int {
        lo: int(DEFAULT_INT.0),
        hi: int(DEFAULT_INT.1),
    };
    let counted = Shape::Int {
        lo: int(1),
        hi: int(DEFAULT_INT.1),
    };
    for field in &mut spec.fields {
        let is_size = |name: &Option<String>| name.as_ref().is_some_and(|name| spec.sizes.contains(name));
        match (&mut field.shape, field.names.as_slice()) {
            (Shape::Tuple(parts), names) if names.len() == parts.len() => {
                for (part, name) in parts.iter_mut().zip(names) {
                    if *part == plain && is_size(name) {
                        *part = counted.clone();
                    }
                }
            }
            (shape, [name]) if *shape == plain && is_size(name) => *shape = counted.clone(),
            _ => {}
        }
    }
    if let Some(cases) = &mut spec.cases {
        sizes_count(cases);
    }
}

struct Builder<'a> {
    src: &'a str,
    base: u32,
    /// `domain V = lo..=hi` as inclusive bounds.
    domains: HashMap<String, (Expr, Expr)>,
    aliases: HashMap<String, Type>,
    spec: InputSpec,
}

impl Builder<'_> {
    fn error(&self, span: Span, message: String) -> SpecError {
        SpecError {
            span: span.shift(self.base),
            message,
        }
    }

    /// `lo..hi` or `lo..=hi` as inclusive bounds.
    fn inclusive(lo: &Expr, hi: &Expr, inclusive: bool) -> (Expr, Expr) {
        let hi = if inclusive {
            hi.clone()
        } else {
            binary(BinOp::Sub, hi.clone(), int(1))
        };
        (lo.clone(), hi)
    }

    /// The bounds of a type used as an array index: a domain or a range.
    fn index(&self, ty: &Type) -> Result<(Expr, Expr), SpecError> {
        match &ty.kind {
            TypeKind::Range { lo, hi, inclusive } => Ok(Self::inclusive(lo, hi, *inclusive)),
            TypeKind::Named { name, args } if args.is_empty() => {
                self.domains.get(&name.text).cloned().ok_or_else(|| {
                    self.error(
                        ty.span,
                        format!("`{}` is not a domain declared above", name.text),
                    )
                })
            }
            _ => Err(self.error(ty.span, "an array is indexed by a domain or a range".into())),
        }
    }

    fn shape(&mut self, ty: &Type) -> Result<Shape, SpecError> {
        let text = ty.span.shift(self.base).text(self.src).to_string();
        let unsupported = |this: &Self| {
            this.error(
                ty.span,
                format!("can't generate a `{text}` yet; pass a generator with `--gen`"),
            )
        };
        match &ty.kind {
            TypeKind::Range { lo, hi, inclusive } => {
                let (lo, hi) = Self::inclusive(lo, hi, *inclusive);
                names_in(&lo, &mut self.spec.sizes);
                names_in(&hi, &mut self.spec.sizes);
                Ok(Shape::Int { lo, hi })
            }
            TypeKind::Named { name, args } if args.is_empty() => {
                match name.text.as_str() {
                    "Int" => {
                        return Ok(Shape::Int {
                            lo: int(DEFAULT_INT.0),
                            hi: int(DEFAULT_INT.1),
                        })
                    }
                    "Char" => return Ok(Shape::Char { lo: b'a', hi: b'z' }),
                    "Bool" => return Ok(Shape::Bool),
                    "Str" => return Ok(Shape::Str),
                    _ => {}
                }
                if let Some((lo, hi)) = self.domains.get(&name.text).cloned() {
                    return Ok(Shape::Int { lo, hi });
                }
                match self.aliases.get(&name.text).cloned() {
                    // `type Perm = [V]V` and `type Tree = [E](u: V, v: V)`
                    // say what the values are, beyond their type.
                    Some(alias) if name.text == "Perm" => self.perm(&alias).ok_or_else(|| unsupported(self)),
                    Some(alias) if name.text == "Tree" => self.tree(&alias).ok_or_else(|| unsupported(self)),
                    Some(alias) => self.shape(&alias),
                    None => Err(unsupported(self)),
                }
            }
            // `Perm<V>` and `Tree<V>`: a permutation of a domain, and the
            // edges of a tree on it.
            TypeKind::Named { name, args } if args.len() == 1 && matches!(name.text.as_str(), "Perm" | "Tree") => {
                let (lo, hi) = self.index(&args[0])?;
                names_in(&lo, &mut self.spec.sizes);
                names_in(&hi, &mut self.spec.sizes);
                Ok(if name.text == "Perm" {
                    let len = binary(BinOp::Add, binary(BinOp::Sub, hi, lo.clone()), int(1));
                    Shape::Perm { lo, len }
                } else {
                    Shape::Tree { lo, hi }
                })
            }
            // `[domain S]Char`: a word that brings its own length.
            TypeKind::OwnDomain { elem, .. } if matches!(self.shape(elem), Ok(Shape::Char { .. })) => Ok(Shape::Str),
            TypeKind::Record(fields) => fields
                .iter()
                .map(|(_, field)| self.shape(field))
                .collect::<Result<_, _>>()
                .map(Shape::Tuple),
            TypeKind::Array { index, elem } => {
                let mut shape = self.shape(elem)?;
                // `[Row, Col]T` is rows of columns.
                for dimension in index.iter().rev() {
                    let (lo, hi) = self.index(dimension)?;
                    let len = binary(BinOp::Add, binary(BinOp::Sub, hi, lo), int(1));
                    names_in(&len, &mut self.spec.sizes);
                    shape = Shape::Array {
                        len,
                        elem: Box::new(shape),
                    };
                }
                Ok(shape)
            }
            TypeKind::Tuple(parts) => parts
                .iter()
                .map(|part| self.shape(part))
                .collect::<Result<_, _>>()
                .map(Shape::Tuple),
            _ => Err(unsupported(self)),
        }
    }

    /// `[V]V` as a permutation of `V`.
    fn perm(&mut self, ty: &Type) -> Option<Shape> {
        let TypeKind::Array { index, .. } = &ty.kind else {
            return None;
        };
        let [dimension] = index.as_slice() else {
            return None;
        };
        let (lo, hi) = self.index(dimension).ok()?;
        names_in(&lo, &mut self.spec.sizes);
        names_in(&hi, &mut self.spec.sizes);
        let len = binary(BinOp::Add, binary(BinOp::Sub, hi, lo.clone()), int(1));
        Some(Shape::Perm { lo, len })
    }

    /// `[E](u: V, v: V)` or `[E](V, V)` as the edges of a tree on `V`.
    fn tree(&mut self, ty: &Type) -> Option<Shape> {
        let TypeKind::Array { elem, .. } = &ty.kind else {
            return None;
        };
        let ends: Vec<&Type> = match &elem.kind {
            TypeKind::Tuple(parts) => parts.iter().collect(),
            TypeKind::Record(fields) => fields.iter().map(|(_, field)| field).collect(),
            _ => return None,
        };
        let [u, v] = ends.as_slice() else {
            return None;
        };
        match (self.shape(u).ok()?, self.shape(v).ok()?) {
            (Shape::Int { lo, hi }, Shape::Int { .. }) => Some(Shape::Tree { lo, hi }),
            _ => None,
        }
    }

    /// The names a pattern gives to the parts of `shape`.
    fn names(pat: &Pattern, shape: &Shape) -> Vec<Option<String>> {
        match (&pat.kind, shape) {
            // The name of anything but a number is only a label:
            // nothing is bound to it.
            (PatternKind::Name(name), shape) if !matches!(shape, Shape::Tuple(_)) => {
                vec![Some(name.clone())]
            }
            (PatternKind::Tuple(pats), Shape::Tuple(shapes)) if pats.len() == shapes.len() => pats
                .iter()
                .zip(shapes)
                .flat_map(|(pat, shape)| Self::names(pat, shape))
                .collect(),
            (_, Shape::Tuple(shapes)) => shapes
                .iter()
                .flat_map(|shape| Self::names(pat, shape).into_iter().map(|_| None))
                .collect(),
            _ => vec![None],
        }
    }
}

fn span_range(span: Span) -> std::ops::Range<usize> {
    span.start as usize..span.end as usize
}

/// True when `code` calls `read()`, outside `//` comments.
fn reads_inside(code: &str) -> bool {
    code.lines().any(|line| {
        let line = line.split("//").next().unwrap_or("");
        line.match_indices("read()").any(|(at, _)| {
            !line[..at].chars().next_back().is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.')
        })
    })
}

fn is_read(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Call { callee, args }
        if args.is_empty() && matches!(&callee.kind, ExprKind::Name(name) if name == "read"))
}

impl InputSpec {
    /// True when the program reads nothing a case can be made from.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.cases.as_ref().is_none_or(|cases| cases.fields.is_empty())
    }

    /// Reads the input a program declares: every top-level
    /// `let pattern: Type = read()`, in order.
    pub fn from_source(src: &str) -> Result<InputSpec, SpecError> {
        let parsed = cpl_syntax::parse_file(src);
        let lines = cpl_diag::LineIndex::new(src);
        let mut builder = Builder {
            src,
            base: 0,
            domains: HashMap::new(),
            aliases: HashMap::new(),
            spec: InputSpec::default(),
        };
        // What is read above `@!multitest`, once the divider is seen.
        let mut above: Option<InputSpec> = None;
        // With an `@interactor`, the hidden test is what its body reads,
        // once per test case; the solution reads only what it is sent.
        let judge: Option<(_, &Block)> = parsed.items.iter().flat_map(|(base, item)| item.stmts.iter().map(move |stmt| (*base, stmt))).find_map(|(base, stmt)| {
            let decorated = stmt.decorators.iter().any(|d| d.name.text == "interactor");
            match &stmt.kind {
                StmtKind::Fn(decl) if decorated => match &decl.body {
                    FnBody::Block(block) => Some((base, block)),
                    _ => None,
                },
                _ => None,
            }
        });
        let mut hidden = Vec::new();
        for (base, item) in &parsed.items {
            builder.base = *base;
            // A `@multitest fn` reads one case: its body is read like
            // the lines below an `@!multitest` divider.
            let stmts = item.stmts.iter().flat_map(|stmt| {
                let multitest = stmt.decorators.iter().any(|d| !d.directive && d.name.text == "multitest");
                let body: &[Stmt] = match &stmt.kind {
                    StmtKind::Fn(decl) if multitest => match &decl.body {
                        FnBody::Block(block) => &block.stmts,
                        _ => &[],
                    },
                    _ => &[],
                };
                std::iter::once(stmt).chain(body).map(|stmt| (stmt, false))
            });
            let stmts = stmts.chain(judge.iter().flat_map(|(_, block)| &block.stmts).map(|stmt| (stmt, true)));
            for (stmt, in_judge) in stmts {
                // The interactor's lines are offset from its own item.
                builder.base = match judge {
                    Some((judge_base, _)) if in_judge => judge_base,
                    _ => *base,
                };
                // Decorated fns (`@brute`, `@checker`, `@interactor`)
                // read for themselves; the divider splits the input.
                if let Some(divider) = stmt.decorators.iter().find(|d| d.name.text == "multitest") {
                    if above.is_none() {
                        let mut setup = std::mem::take(&mut builder.spec);
                        builder.spec.consts = setup.consts.clone();
                        builder.spec.sizes = setup.sizes.clone();
                        setup.until_eof = divider
                            .args
                            .iter()
                            .any(|arg| matches!(&arg.value.kind, ExprKind::Name(name) if name == "until_eof"));
                        above = Some(setup);
                    }
                }
                match &stmt.kind {
                    StmtKind::Const {
                        name,
                        init: Some(init),
                        ..
                    } => {
                        if let Some(value) = eval(init, &builder.spec.consts) {
                            builder.spec.consts.insert(name.text.clone(), value);
                        }
                    }
                    StmtKind::Domain { name, def } => {
                        if let ExprKind::Range {
                            lo: Some(lo),
                            hi: Some(hi),
                            inclusive,
                            step: None,
                        } = &def.kind
                        {
                            let bounds = Builder::inclusive(lo, hi, *inclusive);
                            names_in(&bounds.0, &mut builder.spec.sizes);
                            names_in(&bounds.1, &mut builder.spec.sizes);
                            builder.domains.insert(name.text.clone(), bounds);
                        }
                    }
                    StmtKind::TypeAlias { name, generics, ty } if generics.is_empty() => {
                        builder.aliases.insert(name.text.clone(), ty.clone());
                    }
                    StmtKind::Let {
                        pat,
                        ty,
                        init: Some(init),
                        ..
                    } if is_read(init) => {
                        let Some(ty) = ty else {
                            return Err(builder.error(
                                stmt.span,
                                "this `read()` has no type, so there is nothing to generate from; write `let n: 1..=100 = read()`".into(),
                            ));
                        };
                        let shape = builder.shape(ty)?;
                        let field = Field {
                            names: Builder::names(pat, &shape),
                            shape,
                            line: lines.line_of(stmt.span.shift(builder.base).start) + 1,
                        };
                        match (in_judge, judge.is_some()) {
                            (true, _) => hidden.push(field),
                            (false, false) => builder.spec.fields.push(field),
                            // The solution's own reads are answers
                            // from the interactor, not input.
                            (false, true) => {}
                        }
                    }
                    StmtKind::Fn(_) => {}
                    // A `read()` anywhere else (in an expression, a loop
                    // or a branch) reads input no field describes: a case
                    // made without it would be cut short.
                    _ if (in_judge || judge.is_none()) && reads_inside(&src[span_range(stmt.span.shift(builder.base))]) => {
                        return Err(builder.error(
                            stmt.span,
                            "this `read()` is not a top-level `let name: Type = read()`, so a generated input would leave it out; read it into one first, like `let edges: [m](u: V, v: V) = read()`".into(),
                        ));
                    }
                    _ => {}
                }
            }
            // The interactor is read once, after the file it is in.
            if judge.is_some() && !hidden.is_empty() {
                break;
            }
        }
        if judge.is_some() {
            builder.spec.fields = hidden;
        }
        for (_, item) in &parsed.items {
            for stmt in &item.stmts {
                cpl_syntax::visit::walk_stmt(stmt, &mut Sizes(&mut builder.spec.sizes));
            }
        }
        let consts: HashSet<String> = builder.spec.consts.keys().cloned().collect();
        builder.spec.sizes.retain(|name| !consts.contains(name));
        let mut spec = match above {
            Some(mut setup) => {
                setup.sizes = builder.spec.sizes.clone();
                setup.consts = builder.spec.consts.clone();
                setup.cases = Some(Box::new(builder.spec));
                setup
            }
            None => builder.spec,
        };
        sizes_count(&mut spec);
        Ok(spec)
    }
}

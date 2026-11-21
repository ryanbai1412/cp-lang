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
    /// pair of them followed by one value of each `extra` shape (a weight).
    Tree {
        lo: Expr,
        hi: Expr,
        extra: Vec<Shape>,
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
    /// The names read as a plain `Int`: the program gives them no range,
    /// so the one they are drawn from is a guess `cpl stress` may narrow.
    pub guessed: HashSet<String>,
    /// How far the guessed sizes start above 1.
    pub raised: i128,
    /// The most a guessed size is, when a larger one stopped the program.
    pub capped: Option<i128>,
    /// What the plain `Int` cells of every array read are drawn from,
    /// when some cell stopped the program: `None` is any `Int`.
    pub cells: Option<Box<(i128, i128)>>,
}

/// The most a guessed size's lower bound is raised: a program that still
/// stops with every size at 4 or more is stopped by something else.
pub const MAX_RAISED: i128 = 2;

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
        ExprKind::Char(c) => Some(i128::from(u32::from(*c))),
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
    for field in &spec.fields {
        match (&field.shape, field.names.as_slice()) {
            (Shape::Tuple(parts), names) if names.len() == parts.len() => {
                for (part, name) in parts.iter().zip(names) {
                    if *part == plain {
                        spec.guessed.extend(name.clone());
                    }
                }
            }
            (shape, [name]) if *shape == plain => spec.guessed.extend(name.clone()),
            _ => {}
        }
    }
    if let Some(cases) = &mut spec.cases {
        sizes_count(cases);
    }
    spec.redraw();
}

impl InputSpec {
    /// Gives every guessed name its range: a size counts from
    /// `1 + raised`, anything else is any `Int`.
    fn redraw(&mut self) {
        let lowest = 1 + self.raised;
        for field in &mut self.fields {
            let range = |name: &Option<String>| {
                let name = name.as_ref().filter(|name| self.guessed.contains(*name))?;
                let (lo, hi) = match self.sizes.contains(name) {
                    true => (lowest, self.capped.unwrap_or(DEFAULT_INT.1)),
                    false => DEFAULT_INT,
                };
                Some(Shape::Int {
                    lo: int(lo),
                    hi: int(hi),
                })
            };
            match (&mut field.shape, field.names.as_slice()) {
                (Shape::Tuple(parts), names) if names.len() == parts.len() => {
                    for (part, name) in parts.iter_mut().zip(names) {
                        if let Some(range) = range(name) {
                            *part = range;
                        }
                    }
                }
                (shape, [name]) => {
                    if let Some(range) = range(name) {
                        *shape = range;
                    }
                }
                _ => {}
            }
        }
        if let Some(cases) = &mut self.cases {
            cases.raised = self.raised;
            cases.capped = self.capped;
            cases.redraw();
        }
    }

    /// The guessed names that are not kept small, in the order read.
    pub fn loose(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for spec in std::iter::once(self).chain(self.cases.as_deref()) {
            for name in spec.fields.iter().flat_map(|field| field.names.iter().flatten()) {
                if spec.guessed.contains(name) && !spec.sizes.contains(name) && !names.contains(name) {
                    names.push(name.clone());
                }
            }
        }
        names
    }

    /// The guessed names that are kept small, in the order read.
    pub fn counted(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for spec in std::iter::once(self).chain(self.cases.as_deref()) {
            for name in spec.fields.iter().flat_map(|field| field.names.iter().flatten()) {
                if spec.guessed.contains(name) && spec.sizes.contains(name) && !names.contains(name) {
                    names.push(name.clone());
                }
            }
        }
        names
    }

    /// The same input with `name` drawn as a size: small, and from 1.
    pub fn counting(&self, name: &str) -> InputSpec {
        let mut spec = self.clone();
        spec.sizes.insert(name.to_string());
        if let Some(cases) = &mut spec.cases {
            cases.sizes.insert(name.to_string());
        }
        spec.redraw();
        spec
    }

    /// The same input with the plain `Int` cells of its arrays drawn from
    /// `lo..=hi`, or `None` when it reads no such cell.
    pub fn celled(&self, lo: i128, hi: i128) -> Option<InputSpec> {
        fn redraw(shape: &mut Shape, inside: bool, from: &Shape, to: &Shape) -> bool {
            match shape {
                Shape::Array { elem, .. } => redraw(elem, true, from, to),
                Shape::Tuple(parts) => {
                    parts.iter_mut().fold(false, |any, part| redraw(part, inside, from, to) | any)
                }
                cell if inside && cell == from => {
                    *cell = to.clone();
                    true
                }
                _ => false,
            }
        }
        let range = |(lo, hi): (i128, i128)| Shape::Int {
            lo: int(lo),
            hi: int(hi),
        };
        let (from, to) = (range(self.cells.as_deref().copied().unwrap_or(DEFAULT_INT)), range((lo, hi)));
        let mut spec = self.clone();
        let mut any = false;
        for field in &mut spec.fields {
            any |= redraw(&mut field.shape, false, &from, &to);
        }
        spec.cells = Some(Box::new((lo, hi)));
        if let Some(cases) = &mut spec.cases {
            match cases.celled(lo, hi) {
                Some(inner) => {
                    **cases = inner;
                    any = true;
                }
                None => cases.cells = Some(Box::new((lo, hi))),
            }
        }
        any.then_some(spec)
    }

    /// The same input with every guessed size at most `most`, or `None`
    /// when that leaves them no value.
    pub fn capping(&self, most: i128) -> Option<InputSpec> {
        if most < 1 + self.raised || self.counted().is_empty() {
            return None;
        }
        let mut spec = self.clone();
        spec.capped = Some(most);
        spec.redraw();
        Some(spec)
    }

    /// The same input with every guessed size starting one higher, or
    /// `None` when they are as high as they go.
    pub fn raising(&self) -> Option<InputSpec> {
        let full = self.capped.is_some_and(|most| most <= 1 + self.raised);
        if self.raised >= MAX_RAISED || full || self.counted().is_empty() {
            return None;
        }
        let mut spec = self.clone();
        spec.raised += 1;
        spec.redraw();
        Some(spec)
    }
}

struct Builder<'a> {
    src: &'a str,
    base: u32,
    /// `domain V = lo..=hi` as inclusive bounds.
    domains: HashMap<String, (Expr, Expr)>,
    /// Each alias with the base of the item that declares it, which its
    /// spans are relative to.
    aliases: HashMap<String, (u32, Type)>,
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

    /// The type of one value of a `List<T>` or a one-dimensional array.
    fn cell<'t>(&self, ty: &'t Type) -> Option<&'t Type> {
        match &ty.kind {
            TypeKind::Named { name, args } if name.text == "List" && args.len() == 1 => Some(&args[0]),
            TypeKind::Array { index, elem } if index.len() == 1 => Some(elem),
            _ => None,
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
                let Some((base, alias)) = self.aliases.get(&name.text).cloned() else {
                    return Err(unsupported(self));
                };
                let outer = std::mem::replace(&mut self.base, base);
                // `type Perm = [V]V` and `type Tree = [E](u: V, v: V)`
                // say what the values are, beyond their type.
                let shape = match name.text.as_str() {
                    "Perm" => Ok(self.perm(&alias)),
                    "Tree" => Ok(self.tree(&alias)),
                    _ => self.shape(&alias).map(Some),
                };
                self.base = outer;
                shape?.ok_or_else(|| unsupported(self))
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
                    Shape::Tree { lo, hi, extra: Vec::new() }
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

    /// `[E](u: V, v: V)` or `[E](V, V)` as the edges of a tree on `V`;
    /// fields after the two ends, such as a weight, are made by their type.
    fn tree(&mut self, ty: &Type) -> Option<Shape> {
        let TypeKind::Array { elem, .. } = &ty.kind else {
            return None;
        };
        let ends: Vec<&Type> = match &elem.kind {
            TypeKind::Tuple(parts) => parts.iter().collect(),
            TypeKind::Record(fields) => fields.iter().map(|(_, field)| field).collect(),
            _ => return None,
        };
        let [u, v, rest @ ..] = ends.as_slice() else {
            return None;
        };
        let extra = rest.iter().map(|ty| self.shape(ty).ok()).collect::<Option<_>>()?;
        match (self.shape(u).ok()?, self.shape(v).ok()?) {
            (Shape::Int { lo, hi }, Shape::Int { .. }) => Some(Shape::Tree { lo, hi, extra }),
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

/// `[read() for _ in lo..hi]`: the count `hi - lo` of values it reads.
fn read_count(expr: &Expr) -> Option<Expr> {
    let ExprKind::Comprehension { body, clauses } = &expr.kind else {
        return None;
    };
    let [clause] = clauses.as_slice() else {
        return None;
    };
    let ExprKind::Range {
        lo: Some(lo),
        hi: Some(hi),
        inclusive,
        step: None,
    } = &clause.iter.kind
    else {
        return None;
    };
    if !is_read(body) || !clause.conds.is_empty() {
        return None;
    }
    let (lo, hi) = Builder::inclusive(lo, hi, *inclusive);
    Some(binary(BinOp::Add, binary(BinOp::Sub, hi, lo), int(1)))
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
                        builder.aliases.insert(name.text.clone(), (builder.base, ty.clone()));
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
                    // `let a: List<Int> = [read() for _ in 0..n]`: `n`
                    // values of the list's element type.
                    StmtKind::Let {
                        pat,
                        ty: Some(ty),
                        init: Some(init),
                        ..
                    } if read_count(init).is_some() && builder.cell(ty).is_some() => {
                        let len = read_count(init).expect("tested above");
                        names_in(&len, &mut builder.spec.sizes);
                        let shape = Shape::Array {
                            len,
                            elem: Box::new(builder.shape(builder.cell(ty).expect("tested above"))?),
                        };
                        let field = Field {
                            names: Builder::names(pat, &shape),
                            shape,
                            line: lines.line_of(stmt.span.shift(builder.base).start) + 1,
                        };
                        match (in_judge, judge.is_some()) {
                            (true, _) => hidden.push(field),
                            (false, false) => builder.spec.fields.push(field),
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

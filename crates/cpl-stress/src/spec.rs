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
use cpl_syntax::ast::{
    BinOp, Block, CmpOp, Cond, Expr, ExprKind, FnBody, Pattern, PatternKind, Stmt, StmtKind, Type,
    TypeKind, UnOp,
};
use std::collections::{HashMap, HashSet};

/// The values `Int` takes when a program gives it no range.
pub const DEFAULT_INT: (i128, i128) = (-1_000_000_000, 1_000_000_000);

/// The shape of one value of the input.
#[derive(Clone, PartialEq, Debug)]
pub enum Shape {
    /// An integer in `lo..=hi`; the bounds are evaluated when a case is
    /// made, against the values read so far.
    Int { lo: Expr, hi: Expr },
    /// `len` values of the same shape.
    Array { len: Expr, elem: Box<Shape> },
    /// A tuple, or a record `(u: V, v: V)`: its fields in order. A part
    /// binds the names in `binds` beside it (as a field does), for the
    /// bounds of the parts after it: one pass of a loop that reads `k`,
    /// then `k` values. `binds` is empty when no part binds a name.
    Tuple {
        parts: Vec<Shape>,
        binds: Vec<Vec<Option<String>>>,
    },
    /// One byte in `lo..=hi`: a lowercase letter unless told otherwise.
    Char { lo: u8, hi: u8 },
    /// `0` or `1`.
    Bool,
    /// A word of letters, of any length from 1.
    Str,
    /// Every value of `lo..lo + len` once, in some order.
    Perm { lo: Expr, len: Expr },
    /// The `hi - lo` edges of a tree on the vertices `lo..=hi`, each a
    /// pair of them followed by one value of each `extra` shape (a weight).
    Tree {
        lo: Expr,
        hi: Expr,
        extra: Vec<Shape>,
    },
    /// `len` edges on the vertices `lo..=hi`, each a pair of them followed
    /// by one value of each `extra` shape. No edge is a loop or comes
    /// twice; `kind` says what else holds.
    Graph {
        lo: Expr,
        hi: Expr,
        len: Expr,
        extra: Vec<Shape>,
        kind: GraphKind,
    },
    /// A tag, then the reads of the first arm that takes it, as one
    /// tuple: a query `1 l r` or `2 i`. The tag is bound to `name` while
    /// the arm is read.
    Choice {
        tag: Box<Shape>,
        name: String,
        arms: Vec<Arm>,
    },
    /// `?T` read with `read(none: x)`: a value of `T`, or `none`, which
    /// is written as `x`.
    Maybe { inner: Box<Shape>, none: i128 },
}

/// One way a row goes after its tag: an `if` branch or a `match` arm.
#[derive(Clone, PartialEq, Debug)]
pub struct Arm {
    /// The tags that take it; `None` takes every tag the arms before it
    /// leave.
    pub tags: Option<Vec<i128>>,
    pub parts: Vec<Shape>,
    pub binds: Vec<Vec<Option<String>>>,
}

impl Arm {
    /// The arm that tag `tag` takes; `None` when it takes none and
    /// nothing more is read.
    pub fn taken(arms: &[Arm], tag: i128) -> Option<&Arm> {
        arms.iter()
            .find(|arm| arm.tags.as_ref().is_none_or(|tags| tags.contains(&tag)))
    }
}

impl Shape {
    /// A tuple whose parts bind no names.
    pub fn tuple(parts: Vec<Shape>) -> Shape {
        Shape::Tuple {
            parts,
            binds: Vec::new(),
        }
    }
}

/// What a `Shape::Graph` promises beyond being simple.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GraphKind {
    /// Undirected, nothing more.
    Simple,
    /// Undirected, and every vertex reaches every other.
    Connected,
    /// Directed, with no cycle.
    Dag,
    /// Directed, nothing more: `u v` and `v u` are two arcs.
    Digraph,
}

impl GraphKind {
    fn named(name: &str) -> Option<GraphKind> {
        match name {
            "SimpleGraph" => Some(GraphKind::Simple),
            "ConnectedGraph" => Some(GraphKind::Connected),
            "Dag" => Some(GraphKind::Dag),
            "Digraph" => Some(GraphKind::Digraph),
            _ => None,
        }
    }

    pub fn directed(self) -> bool {
        matches!(self, GraphKind::Dag | GraphKind::Digraph)
    }
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

pub(crate) fn names_in(expr: &Expr, out: &mut HashSet<String>) {
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
                            if let Some(lo) = lo {
                                names_in(lo, self.0);
                            }
                            if let Some(hi) = hi {
                                names_in(hi, self.0);
                            }
                        }
                        _ => self.of_type(dimension),
                    }
                }
                self.of_type(elem);
            }
            TypeKind::Named { args, .. } | TypeKind::Tuple(args) => {
                args.iter().for_each(|arg| self.of_type(arg))
            }
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
            (Shape::Tuple { parts, .. }, names) if names.len() == parts.len() => {
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
                (Shape::Tuple { parts, .. }, names) if names.len() == parts.len() => {
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
            for name in spec
                .fields
                .iter()
                .flat_map(|field| field.names.iter().flatten())
            {
                if spec.guessed.contains(name)
                    && !spec.sizes.contains(name)
                    && !names.contains(name)
                {
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
            for name in spec
                .fields
                .iter()
                .flat_map(|field| field.names.iter().flatten())
            {
                if spec.guessed.contains(name) && spec.sizes.contains(name) && !names.contains(name)
                {
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
                Shape::Tuple { parts, .. } => parts
                    .iter_mut()
                    .fold(false, |any, part| redraw(part, inside, from, to) | any),
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
        let (from, to) = (
            range(self.cells.as_deref().copied().unwrap_or(DEFAULT_INT)),
            range((lo, hi)),
        );
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
    /// Each record type `read()` reads field by field: a struct's fields,
    /// or an `@tags` enum's variants each with the letter it is written as.
    records: HashMap<String, (u32, Record)>,
    /// How the `read()` being built writes `none`, from `read(none: x)`.
    none: Option<i128>,
    spec: InputSpec,
}

#[derive(Clone)]
enum Record {
    Struct(Vec<Type>),
    Enum(Vec<(u8, Vec<Type>)>),
}

impl Record {
    /// `@tags("R", "C")` above an enum: one letter per variant, in order.
    /// Tags longer than a letter are not generated yet.
    fn tagged(stmt: &Stmt, decl: &cpl_syntax::ast::EnumDecl) -> Option<Record> {
        let tags = stmt.decorators.iter().find(|d| d.name.text == "tags")?;
        if tags.args.len() != decl.variants.len() {
            return None;
        }
        let letters = tags.args.iter().map(|arg| match &arg.value.kind {
            ExprKind::Str(pieces) => match pieces.as_slice() {
                [cpl_syntax::ast::StrPiece::Text(text)] if text.len() == 1 && text.is_ascii() => {
                    Some(text.as_bytes()[0])
                }
                _ => None,
            },
            ExprKind::Char(c) if c.is_ascii() => Some(*c as u8),
            _ => None,
        });
        let variants = letters
            .zip(&decl.variants)
            .map(|(letter, variant)| {
                Some((
                    letter?,
                    variant.fields.iter().map(|(_, ty)| ty.clone()).collect(),
                ))
            })
            .collect::<Option<_>>()?;
        Some(Record::Enum(variants))
    }
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
            TypeKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
            } => Ok(Self::inclusive(lo, hi, *inclusive)),
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
            TypeKind::Named { name, args } if name.text == "List" && args.len() == 1 => {
                Some(&args[0])
            }
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
            TypeKind::Optional(inner) => {
                let none = self.none.ok_or_else(|| unsupported(self))?;
                match self.shape(inner)? {
                    inner @ Shape::Int { .. } => Ok(Shape::Maybe {
                        inner: Box::new(inner),
                        none,
                    }),
                    _ => Err(unsupported(self)),
                }
            }
            TypeKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
            } => {
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
                if let Some((base, record)) = self.records.get(&name.text).cloned() {
                    let outer = std::mem::replace(&mut self.base, base);
                    let shape = self.record(&name.text, &record);
                    self.base = outer;
                    return shape;
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
                    other => match GraphKind::named(other) {
                        Some(kind) => Ok(self.graph(&alias, kind)),
                        None => self.shape(&alias).map(Some),
                    },
                };
                self.base = outer;
                shape?.ok_or_else(|| unsupported(self))
            }
            // `Perm<V>` and `Tree<V>`: a permutation of a domain, and the
            // edges of a tree on it.
            TypeKind::Named { name, args }
                if args.len() == 1 && matches!(name.text.as_str(), "Perm" | "Tree") =>
            {
                let (lo, hi) = self.index(&args[0])?;
                names_in(&lo, &mut self.spec.sizes);
                names_in(&hi, &mut self.spec.sizes);
                Ok(if name.text == "Perm" {
                    let len = binary(BinOp::Add, binary(BinOp::Sub, hi, lo.clone()), int(1));
                    Shape::Perm { lo, len }
                } else {
                    Shape::Tree {
                        lo,
                        hi,
                        extra: Vec::new(),
                    }
                })
            }
            // `SimpleGraph<V, E>`, `ConnectedGraph<V, E>` and `Dag<V, E>`:
            // one edge on `V` for each value of `E`.
            TypeKind::Named { name, args }
                if args.len() == 2 && GraphKind::named(&name.text).is_some() =>
            {
                let kind = GraphKind::named(&name.text).expect("a graph name");
                let (lo, hi) = self.index(&args[0])?;
                let (first, last) = self.index(&args[1])?;
                let len = binary(BinOp::Add, binary(BinOp::Sub, last, first), int(1));
                for expr in [&lo, &hi, &len] {
                    names_in(expr, &mut self.spec.sizes);
                }
                Ok(Shape::Graph {
                    lo,
                    hi,
                    len,
                    extra: Vec::new(),
                    kind,
                })
            }
            // `[domain S]Char`: a word that brings its own length.
            TypeKind::OwnDomain { elem, .. }
                if matches!(self.shape(elem), Ok(Shape::Char { .. })) =>
            {
                Ok(Shape::Str)
            }
            TypeKind::Record(fields) => fields
                .iter()
                .map(|(_, field)| self.shape(field))
                .collect::<Result<_, _>>()
                .map(Shape::tuple),
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
                .map(Shape::tuple),
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
        let (_, lo, hi, extra) = self.edges(ty)?;
        Some(Shape::Tree { lo, hi, extra })
    }

    /// `[E](u: V, v: V, ..)` as `kind` of graph on `V` with an edge for
    /// each value of `E`.
    fn graph(&mut self, ty: &Type, kind: GraphKind) -> Option<Shape> {
        let (len, lo, hi, extra) = self.edges(ty)?;
        names_in(&len, &mut self.spec.sizes);
        Some(Shape::Graph {
            lo,
            hi,
            len,
            extra,
            kind,
        })
    }

    /// An edge list `[E](u: V, v: V, ..)`: its length, the ends' range
    /// and the shapes of the fields after the ends.
    fn edges(&mut self, ty: &Type) -> Option<(Expr, Expr, Expr, Vec<Shape>)> {
        let TypeKind::Array { index, elem } = &ty.kind else {
            return None;
        };
        let [dimension] = index.as_slice() else {
            return None;
        };
        let (first, last) = self.index(dimension).ok()?;
        let len = binary(BinOp::Add, binary(BinOp::Sub, last, first), int(1));
        let ends: Vec<&Type> = match &elem.kind {
            TypeKind::Tuple(parts) => parts.iter().collect(),
            TypeKind::Record(fields) => fields.iter().map(|(_, field)| field).collect(),
            _ => return None,
        };
        let [u, v, rest @ ..] = ends.as_slice() else {
            return None;
        };
        let extra = rest
            .iter()
            .map(|ty| self.shape(ty).ok())
            .collect::<Option<_>>()?;
        match (self.shape(u).ok()?, self.shape(v).ok()?) {
            (Shape::Int { lo, hi }, Shape::Int { .. }) => Some((len, lo, hi, extra)),
            _ => None,
        }
    }

    /// `for _ in 0..q:` whose body reads only through `let x: T = read()`
    /// lines (or loops like this one) with types that use nothing bound
    /// inside the loop: `q` rows of what one pass reads. `None` for any
    /// other loop, whose reads a field cannot describe.
    fn pass(&mut self, stmt: &Stmt, outer: &HashSet<String>) -> Result<Option<Shape>, SpecError> {
        let StmtKind::For {
            pat, iter, body, ..
        } = &stmt.kind
        else {
            return Ok(None);
        };
        let Some((lo, hi)) = self.over(iter) else {
            return Ok(None);
        };
        let mut used = HashSet::new();
        names_in(&lo, &mut used);
        names_in(&hi, &mut used);
        if !used.is_disjoint(outer) {
            return Ok(None);
        }
        let mut inside = outer.clone();
        bind_names(pat, &mut inside);
        let mut parts = Vec::new();
        let mut binds = Vec::new();
        if !self.rows(&body.stmts, &mut inside, &mut parts, &mut binds)? {
            return Ok(None);
        }
        let elem = match parts.len() {
            0 => return Ok(None),
            1 => parts.pop().expect("one part"),
            _ if binds.iter().all(Vec::is_empty) => Shape::tuple(parts),
            _ => Shape::Tuple { parts, binds },
        };
        names_in(&lo, &mut self.spec.sizes);
        names_in(&hi, &mut self.spec.sizes);
        Ok(Some(Shape::Array {
            len: binary(BinOp::Add, binary(BinOp::Sub, hi, lo), int(1)),
            elem: Box::new(elem),
        }))
    }

    fn reads(&self, stmt: &Stmt) -> bool {
        reads_inside(&self.src[span_range(stmt.span.shift(self.base))])
    }

    /// The reads of `stmts`, one pass of a loop or one arm of a choice,
    /// added to `parts`. False when a field cannot describe them.
    fn rows(
        &mut self,
        stmts: &[Stmt],
        inside: &mut HashSet<String>,
        parts: &mut Vec<Shape>,
        binds: &mut Vec<Vec<Option<String>>>,
    ) -> Result<bool, SpecError> {
        let last = stmts.iter().rposition(|stmt| self.reads(stmt));
        for (at, stmt) in stmts.iter().enumerate() {
            let mut jumps = Jumps::default();
            cpl_syntax::visit::walk_stmt(stmt, &mut jumps);
            if jumps.leaves || (jumps.skips && last.is_some_and(|last| at < last)) {
                return Ok(false);
            }
            match &stmt.kind {
                StmtKind::Let {
                    pat,
                    ty: Some(ty),
                    init: Some(init),
                    ..
                } if read_none(init).is_some() => {
                    let mut used = HashSet::new();
                    Sizes(&mut used).of_type(ty);
                    if !used.is_disjoint(inside) {
                        return Ok(false);
                    }
                    let shape = self.shape_written(ty, read_none(init).flatten())?;
                    // A name read here is bound beside its value, so the
                    // reads after it may use it: `k`, then `[k]Int`.
                    let names = Self::names(pat, &shape);
                    for name in names.iter().flatten() {
                        inside.remove(name);
                    }
                    let mut all = HashSet::new();
                    bind_names(pat, &mut all);
                    inside.extend(
                        all.into_iter()
                            .filter(|name| !names.contains(&Some(name.clone()))),
                    );
                    binds.push(names);
                    parts.push(shape);
                }
                StmtKind::Domain { name, def } => {
                    let (bounds, read) = match &def.kind {
                        ExprKind::Range {
                            lo: Some(lo),
                            hi: Some(hi),
                            inclusive,
                            step: None,
                        } => (Self::inclusive(lo, hi, *inclusive), None),
                        _ => match read_size(def) {
                            Some((lo, hi, inclusive)) => (
                                counted(&name.text),
                                Some(Self::inclusive(lo, hi, inclusive)),
                            ),
                            None => return Ok(false),
                        },
                    };
                    let (lo, hi) = read.as_ref().unwrap_or(&bounds);
                    let mut used = HashSet::new();
                    names_in(lo, &mut used);
                    names_in(hi, &mut used);
                    if !used.is_disjoint(inside) {
                        return Ok(false);
                    }
                    self.spec.sizes.extend(used);
                    if let Some((lo, hi)) = read {
                        self.spec.sizes.insert(name.text.clone());
                        binds.push(vec![Some(name.text.clone())]);
                        parts.push(Shape::Int { lo, hi });
                    }
                    self.domains.insert(name.text.clone(), bounds);
                }
                StmtKind::For { .. } if self.reads(stmt) => match self.pass(stmt, inside)? {
                    Some(shape) => {
                        binds.push(Vec::new());
                        parts.push(shape);
                    }
                    None => return Ok(false),
                },
                StmtKind::Expr(expr) if self.reads(stmt) => {
                    if !self.choice(expr, inside, parts, binds)? {
                        return Ok(false);
                    }
                }
                _ if self.reads(stmt) => return Ok(false),
                StmtKind::Let { pat, .. } => bind_names(pat, inside),
                _ => {}
            }
        }
        Ok(true)
    }

    /// An `if` or `match` on the tag just read, each branch reading its
    /// own values: `if op == 1: let (l, r) = read() else: let i = read()`.
    fn choice(
        &mut self,
        expr: &Expr,
        inside: &mut HashSet<String>,
        parts: &mut Vec<Shape>,
        binds: &mut Vec<Vec<Option<String>>>,
    ) -> Result<bool, SpecError> {
        let Some([Some(name)]) = binds.last().map(Vec::as_slice) else {
            return Ok(false);
        };
        let name = name.clone();
        if !matches!(
            parts.last(),
            Some(Shape::Int { .. } | Shape::Char { .. } | Shape::Bool)
        ) {
            return Ok(false);
        }
        let consts = self.spec.consts.clone();
        let literal = |expr: &Expr| eval(expr, &consts);
        // The tags `x == 1 or x == 2` takes, with `x` the tag.
        fn tags(
            cond: &Expr,
            name: &str,
            literal: &dyn Fn(&Expr) -> Option<i128>,
        ) -> Option<(Vec<i128>, bool)> {
            match &cond.kind {
                ExprKind::Paren(inner) => tags(inner, name, literal),
                ExprKind::Binary {
                    op: BinOp::Or,
                    lhs,
                    rhs,
                    ..
                } => {
                    let (mut a, false) = tags(lhs, name, literal)? else {
                        return None;
                    };
                    let (b, false) = tags(rhs, name, literal)? else {
                        return None;
                    };
                    a.extend(b);
                    Some((a, false))
                }
                ExprKind::Compare { first, rest } => {
                    let [(op @ (CmpOp::Eq | CmpOp::Ne), other)] = rest.as_slice() else {
                        return None;
                    };
                    let is_tag = |e: &Expr| matches!(&e.kind, ExprKind::Name(n) if n == name);
                    let value = match (is_tag(first), is_tag(other)) {
                        (true, false) => literal(other)?,
                        (false, true) => literal(first)?,
                        _ => return None,
                    };
                    Some((vec![value], *op == CmpOp::Ne))
                }
                _ => None,
            }
        }
        let mut ways: Vec<(Option<Vec<i128>>, Option<&Block>)> = Vec::new();
        match &expr.kind {
            ExprKind::If {
                branches,
                otherwise,
            } => {
                for (at, (cond, block)) in branches.iter().enumerate() {
                    let Cond::Expr(cond) = cond else {
                        return Ok(false);
                    };
                    match tags(cond, &name, &literal) {
                        Some((values, false)) => ways.push((Some(values), Some(block))),
                        // `if x != 1: A else: B` is `B` for 1, else `A`.
                        Some((values, true)) if at == 0 && branches.len() == 1 => {
                            ways.push((Some(values), otherwise.as_ref()));
                            ways.push((None, Some(block)));
                        }
                        _ => return Ok(false),
                    }
                }
                if !matches!(ways.last(), Some((None, _))) {
                    ways.push((None, otherwise.as_ref()));
                }
            }
            ExprKind::Match { scrutinee, arms } if matches!(&scrutinee.kind, ExprKind::Name(n) if *n == name) =>
            {
                fn values(
                    pat: &Pattern,
                    literal: &dyn Fn(&Expr) -> Option<i128>,
                ) -> Option<Option<Vec<i128>>> {
                    match &pat.kind {
                        PatternKind::Wildcard | PatternKind::Name(_) => Some(None),
                        PatternKind::Literal(expr) => Some(Some(vec![literal(expr)?])),
                        PatternKind::Or(pats) => {
                            let mut all = Vec::new();
                            for pat in pats {
                                all.extend(values(pat, literal)??);
                            }
                            Some(Some(all))
                        }
                        _ => None,
                    }
                }
                for arm in arms {
                    if arm.guard.is_some() {
                        return Ok(false);
                    }
                    let Some(tags) = values(&arm.pat, &literal) else {
                        return Ok(false);
                    };
                    ways.push((tags, Some(&arm.body)));
                }
            }
            _ => return Ok(false),
        }
        let tag = parts.pop().expect("tested above");
        binds.pop();
        let mut arms = Vec::new();
        for (tags, block) in ways {
            let mut arm = Arm {
                tags,
                parts: Vec::new(),
                binds: Vec::new(),
            };
            if let Some(block) = block {
                let mut inside = inside.clone();
                if !self.rows(&block.stmts, &mut inside, &mut arm.parts, &mut arm.binds)? {
                    return Ok(false);
                }
            }
            arms.push(arm);
        }
        // An `Int` tag read with no range takes the values its arms test,
        // and one past them for an arm that takes the rest: the statement
        // numbers its queries, and a far-off tag only reads as noise.
        let tag = match tag {
            Shape::Int { lo, hi }
                if eval(&lo, &consts) == Some(DEFAULT_INT.0)
                    && eval(&hi, &consts) == Some(DEFAULT_INT.1) =>
            {
                let named: Vec<i128> = arms
                    .iter()
                    .flat_map(|arm| arm.tags.iter().flatten().copied())
                    .collect();
                let rest = arms.iter().any(|arm| arm.tags.is_none());
                match (named.iter().min(), named.iter().max()) {
                    (Some(&min), Some(&max)) => Shape::Int {
                        lo: int(min),
                        hi: int(max + i128::from(rest)),
                    },
                    _ => Shape::Int { lo, hi },
                }
            }
            tag => tag,
        };
        inside.insert(name.clone());
        parts.push(Shape::Choice {
            tag: Box::new(tag),
            name,
            arms,
        });
        binds.push(Vec::new());
        Ok(true)
    }

    /// The shape of `ty` read with `none` written as `none`.
    fn shape_written(&mut self, ty: &Type, none: Option<i128>) -> Result<Shape, SpecError> {
        let outer = std::mem::replace(&mut self.none, none);
        let shape = self.shape(ty);
        self.none = outer;
        shape
    }

    /// A struct is its fields in order; an `@tags` enum is a choice on
    /// the letter its variant is written as, then that variant's fields.
    fn record(&mut self, name: &str, record: &Record) -> Result<Shape, SpecError> {
        let parts = |this: &mut Self, types: &[Type]| -> Result<Vec<Shape>, SpecError> {
            types.iter().map(|ty| this.shape(ty)).collect()
        };
        match record {
            Record::Struct(fields) => {
                let parts = parts(self, fields)?;
                let binds = vec![Vec::new(); parts.len()];
                Ok(Shape::Tuple { parts, binds })
            }
            Record::Enum(variants) => {
                let mut arms = Vec::new();
                for (tag, fields) in variants {
                    let parts = parts(self, fields)?;
                    let binds = vec![Vec::new(); parts.len()];
                    arms.push(Arm {
                        tags: Some(vec![i128::from(*tag)]),
                        parts,
                        binds,
                    });
                }
                let letters = variants.iter().map(|(tag, _)| *tag);
                let (lo, hi) = (letters.clone().min(), letters.max());
                let (Some(lo), Some(hi)) = (lo, hi) else {
                    return Err(
                        self.error(Span::new(0, 0), format!("`{name}` has no variants to read"))
                    );
                };
                Ok(Shape::Choice {
                    tag: Box::new(Shape::Char { lo, hi }),
                    name: format!("{name}.tag"),
                    arms,
                })
            }
        }
    }

    /// The first and last value a loop goes over: `lo..hi`, `lo..=hi`
    /// or a domain declared above.
    fn over(&self, iter: &Expr) -> Option<(Expr, Expr)> {
        match &iter.kind {
            ExprKind::Paren(inner) => self.over(inner),
            ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step: None,
            } => Some(Builder::inclusive(lo, hi, *inclusive)),
            ExprKind::Name(name) => self.domains.get(name).cloned(),
            _ => None,
        }
    }

    /// The names a pattern gives to the parts of `shape`.
    fn names(pat: &Pattern, shape: &Shape) -> Vec<Option<String>> {
        match (&pat.kind, shape) {
            // The name of anything but a number is only a label:
            // nothing is bound to it.
            (PatternKind::Name(name), shape) if !matches!(shape, Shape::Tuple { .. }) => {
                vec![Some(name.clone())]
            }
            (PatternKind::Tuple(pats), Shape::Tuple { parts: shapes, .. })
                if pats.len() == shapes.len() =>
            {
                pats.iter()
                    .zip(shapes)
                    .flat_map(|(pat, shape)| Self::names(pat, shape))
                    .collect()
            }
            (_, Shape::Tuple { parts: shapes, .. }) => shapes
                .iter()
                .flat_map(|shape| Self::names(pat, shape).into_iter().map(|_| None))
                .collect(),
            _ => vec![None],
        }
    }
}

fn bind_names(pat: &Pattern, out: &mut HashSet<String>) {
    match &pat.kind {
        PatternKind::Name(name) => {
            out.insert(name.clone());
        }
        PatternKind::Domain(name) => {
            out.insert(name.text.clone());
        }
        PatternKind::Tuple(pats)
        | PatternKind::Or(pats)
        | PatternKind::Variant { args: pats, .. } => {
            pats.iter().for_each(|pat| bind_names(pat, out))
        }
        _ => {}
    }
}

/// Whether a statement can end its loop early (`break`, `return`) or
/// skip the rest of a pass (`continue`).
#[derive(Default)]
struct Jumps {
    leaves: bool,
    skips: bool,
}

impl cpl_syntax::visit::Visitor for Jumps {
    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Break(_) | StmtKind::Return(_) => self.leaves = true,
            StmtKind::Continue(_) => self.skips = true,
            _ => {}
        }
    }
}

fn span_range(span: Span) -> std::ops::Range<usize> {
    span.start as usize..span.end as usize
}

/// True when `code` calls `read()`, or `read` with named options such
/// as `read(gen: f)`, outside `//` comments.
fn reads_inside(code: &str) -> bool {
    read_calls(code).next().is_some()
}

/// True when `code` calls `read` with named options (`gen:`, `none:`).
fn reads_with_options(code: &str) -> bool {
    read_calls(code).any(|options| options)
}

/// Each call of the built-in `read` in `code`: whether it has options.
fn read_calls(code: &str) -> impl Iterator<Item = bool> + '_ {
    code.lines().flat_map(|line| {
        let line = line.split("//").next().unwrap_or("");
        line.match_indices("read(").filter_map(move |(at, _)| {
            let before = line[..at].chars().next_back();
            if before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.') {
                return None;
            }
            let rest = line[at + "read(".len()..].trim_start();
            if rest.starts_with(')') {
                return Some(false);
            }
            let name = rest
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .map_or(rest, |end| &rest[..end]);
            (!name.is_empty() && rest[name.len()..].trim_start().starts_with(':')).then_some(true)
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

/// The range of `read(1..=n)`: a size read from the input, as in
/// `domain V = read(1..=2e5)`.
fn read_size(expr: &Expr) -> Option<(&Expr, &Expr, bool)> {
    let ExprKind::Call { callee, args } = &expr.kind else {
        return None;
    };
    let [arg] = args.as_slice() else {
        return None;
    };
    if !matches!(&callee.kind, ExprKind::Name(name) if name == "read") || arg.name.is_some() {
        return None;
    }
    match &arg.value.kind {
        ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
            step: None,
        } => Some((lo, hi, *inclusive)),
        _ => None,
    }
}

/// A domain of `count` positions, `0..count`.
fn counted(count: &str) -> (Expr, Expr) {
    let count = Expr {
        kind: ExprKind::Name(count.into()),
        span: Span::new(0, 0),
    };
    (int(0), binary(BinOp::Sub, count, int(1)))
}

/// `read()` as `Some(None)`, and `read(none: x)` with a number `x`
/// (or a string that is one) as `Some(Some(x))`.
fn read_none(expr: &Expr) -> Option<Option<i128>> {
    if is_read(expr) {
        return Some(None);
    }
    let ExprKind::Call { callee, args } = &expr.kind else {
        return None;
    };
    let [arg] = args.as_slice() else {
        return None;
    };
    if !matches!(&callee.kind, ExprKind::Name(name) if name == "read")
        || arg.name.as_ref().is_none_or(|name| name.text != "none")
    {
        return None;
    }
    let none = match &arg.value.kind {
        ExprKind::Str(pieces) => match pieces.as_slice() {
            [cpl_syntax::ast::StrPiece::Text(text)] => text.trim().parse().ok(),
            _ => None,
        },
        _ => eval(&arg.value, &Env::new()),
    };
    Some(Some(none?))
}

fn is_read(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Call { callee, args }
        if args.is_empty() && matches!(&callee.kind, ExprKind::Name(name) if name == "read"))
}

impl InputSpec {
    /// True when the program reads nothing a case can be made from.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
            && self
                .cases
                .as_ref()
                .is_none_or(|cases| cases.fields.is_empty())
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
            records: HashMap::new(),
            none: None,
            spec: InputSpec::default(),
        };
        // What is read above `@!multitest`, once the divider is seen.
        let mut above: Option<InputSpec> = None;
        // With an `@interactor`, the hidden test is what its body reads,
        // once per test case; the solution reads only what it is sent.
        let judge: Option<(_, &Block)> = parsed
            .items
            .iter()
            .flat_map(|(base, item)| item.stmts.iter().map(move |stmt| (*base, stmt)))
            .find_map(|(base, stmt)| {
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
                let multitest = stmt
                    .decorators
                    .iter()
                    .any(|d| !d.directive && d.name.text == "multitest");
                let body: &[Stmt] = match &stmt.kind {
                    StmtKind::Fn(decl) if multitest => match &decl.body {
                        FnBody::Block(block) => &block.stmts,
                        _ => &[],
                    },
                    _ => &[],
                };
                std::iter::once(stmt).chain(body).map(|stmt| (stmt, false))
            });
            let stmts = stmts.chain(
                judge
                    .iter()
                    .flat_map(|(_, block)| &block.stmts)
                    .map(|stmt| (stmt, true)),
            );
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
                        } else if let Some((lo, hi, inclusive)) = read_size(def) {
                            let (lo, hi) = Builder::inclusive(lo, hi, inclusive);
                            names_in(&lo, &mut builder.spec.sizes);
                            names_in(&hi, &mut builder.spec.sizes);
                            builder.spec.sizes.insert(name.text.clone());
                            builder
                                .domains
                                .insert(name.text.clone(), counted(&name.text));
                            let field = Field {
                                names: vec![Some(name.text.clone())],
                                shape: Shape::Int { lo, hi },
                                line: lines.line_of(stmt.span.shift(builder.base).start) + 1,
                            };
                            match (in_judge, judge.is_some()) {
                                (true, _) => hidden.push(field),
                                (false, false) => builder.spec.fields.push(field),
                                (false, true) => {}
                            }
                        }
                    }
                    StmtKind::TypeAlias { name, generics, ty } if generics.is_empty() => {
                        builder
                            .aliases
                            .insert(name.text.clone(), (builder.base, ty.clone()));
                    }
                    StmtKind::Struct(decl) if decl.generics.is_empty() => {
                        let fields = decl.fields.iter().map(|f| f.ty.clone()).collect();
                        builder.records.insert(
                            decl.name.text.clone(),
                            (builder.base, Record::Struct(fields)),
                        );
                    }
                    StmtKind::Enum(decl) if decl.generics.is_empty() => {
                        if let Some(record) = Record::tagged(stmt, decl) {
                            builder
                                .records
                                .insert(decl.name.text.clone(), (builder.base, record));
                        }
                    }
                    StmtKind::Let {
                        pat,
                        ty,
                        init: Some(init),
                        ..
                    } if read_none(init).is_some() => {
                        let Some(ty) = ty else {
                            return Err(builder.error(
                                stmt.span,
                                "this `read()` has no type, so there is nothing to generate from; write `let n: 1..=100 = read()`".into(),
                            ));
                        };
                        let shape = builder.shape_written(ty, read_none(init).flatten())?;
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
                    StmtKind::For { .. }
                        if (in_judge || judge.is_none())
                            && reads_inside(&src[span_range(stmt.span.shift(builder.base))]) =>
                    {
                        let Some(shape) = builder.pass(stmt, &HashSet::new())? else {
                            return Err(builder.error(
                                stmt.span,
                                "this loop reads more than `let name: Type = read()` lines whose types use only names from above it, so a generated input would leave its reads out; read them into one first, like `let edges: [m](u: V, v: V) = read()`".into(),
                            ));
                        };
                        let field = Field {
                            names: vec![None],
                            shape,
                            line: lines.line_of(stmt.span.shift(builder.base).start) + 1,
                        };
                        match in_judge {
                            true => hidden.push(field),
                            false => builder.spec.fields.push(field),
                        }
                    }
                    // A fn the program calls reads input no field
                    // describes; `@brute`, `@checker` and the like read for
                    // themselves.
                    StmtKind::Fn(_)
                        if (in_judge || judge.is_none())
                            && stmt
                                .decorators
                                .iter()
                                .all(|d| matches!(d.name.text.as_str(), "memo" | "allow"))
                            && reads_inside(&src[span_range(stmt.span.shift(builder.base))]) =>
                    {
                        return Err(builder.error(
                            stmt.span,
                            "this fn reads input, so a generated input would leave its reads out; read them into a top-level `let name: Type = read()` and pass the values in".into(),
                        ));
                    }
                    StmtKind::Fn(_) => {}
                    // A `read()` anywhere else (in an expression, a loop
                    // or a branch) reads input no field describes: a case
                    // made without it would be cut short.
                    _ if (in_judge || judge.is_none())
                        && reads_with_options(&src[span_range(stmt.span.shift(builder.base))]) =>
                    {
                        return Err(builder.error(
                            stmt.span,
                            "`cpl stress` can't make a `read(gen: ..)` or `read(none: ..)` yet, so a generated input would leave it out; write a `@gen fn` that prints a whole input, or pass `--gen <program>`".into(),
                        ));
                    }
                    _ if (in_judge || judge.is_none())
                        && reads_inside(&src[span_range(stmt.span.shift(builder.base))]) =>
                    {
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

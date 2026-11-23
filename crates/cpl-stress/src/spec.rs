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
    AssignOp, BinOp, Block, CmpOp, Cond, Expr, ExprKind, FnBody, Pattern, PatternKind, Stmt,
    StmtKind, StrPiece, Type, TypeKind, UnOp,
};
use std::collections::{HashMap, HashSet};

mod build;
mod syntax;
use build::*;
use syntax::*;

/// The values `Int` takes when a program gives it no range.
pub const DEFAULT_INT: (i128, i128) = (-1_000_000_000, 1_000_000_000);

/// The shape of one value of the input.
#[derive(Clone, PartialEq, Debug)]
pub enum Shape {
    /// An integer in `lo..=hi`; the bounds are evaluated when a case is
    /// made, against the values read so far.
    Int { lo: Expr, hi: Expr },
    /// `len` values of the same shape. With `index`, each value is made
    /// with that name bound to `lo` plus its position, as the loop
    /// `for r in 1..=n: let row: [r]Int = read()` binds `r`.
    Array {
        len: Expr,
        elem: Box<Shape>,
        index: Option<(String, Expr)>,
    },
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
    /// A word over `alphabet` (lowercase letters when `None`), its length
    /// in `len` (from 1 when `None`).
    Str {
        alphabet: Option<std::sync::Arc<[u8]>>,
        len: Option<(Expr, Expr)>,
    },
    /// One of these words: a `Str` tag a choice compares with literals.
    Words(std::sync::Arc<[String]>),
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
    /// `read(gen: f)`: what `f()` makes, printed by `program`, which
    /// reads the input above it first.
    Gen {
        func: String,
        program: std::sync::Arc<cpl_ir::Program>,
    },
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
    /// A plain `Str`: lowercase letters, any length from 1.
    pub const WORD: Shape = Shape::Str {
        alphabet: None,
        len: None,
    };

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
/// `+ - * / % ** << >>`, `min` and `max`.
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
                BinOp::Rem if b != 0 => Some(a.rem_euclid(b)),
                BinOp::Pow => a.checked_pow(u32::try_from(b).ok()?),
                BinOp::Shl if (0..127).contains(&b) => a.checked_mul(1 << b),
                BinOp::Shr => Some(a >> u32::try_from(b).ok()?.min(127)),
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

impl InputSpec {
    /// True when the program reads nothing a case can be made from.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
            && self
                .cases
                .as_ref()
                .is_none_or(|cases| cases.fields.is_empty())
    }

    /// Narrows the read of `name` to `value`, so every case made reads it:
    /// a lint's witness. False when the program reads no such integer.
    pub fn pin(&mut self, name: &str, value: i128) -> bool {
        let fixed = || Shape::Int {
            lo: int(value),
            hi: int(value),
        };
        for field in &mut self.fields {
            let Some(at) = field.names.iter().position(|n| n.as_deref() == Some(name)) else {
                continue;
            };
            let count = field.names.len();
            let part = match &mut field.shape {
                shape @ Shape::Int { .. } if count == 1 => Some(shape),
                Shape::Tuple { parts, .. } if parts.len() == count => parts.get_mut(at),
                _ => None,
            };
            if let Some(part) = part.filter(|p| matches!(p, Shape::Int { .. })) {
                *part = fixed();
                return true;
            }
        }
        self.cases
            .as_mut()
            .is_some_and(|cases| cases.pin(name, value))
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
            values: values(src, &parsed.items),
            elems: elems(&parsed.items),
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
        // A drawn value reads the input above it, which a test case
        // can't give it yet.
        let multitest = parsed.items.iter().any(|(_, item)| {
            item.stmts
                .iter()
                .any(|stmt| stmt.decorators.iter().any(|d| d.name.text == "multitest"))
        });
        let readers = readers(src, &parsed.items);
        // Where the last top-level line that reads ends: a branch there
        // may take the fields above it into its arms, since nothing
        // after it reads by their names.
        let last_read = parsed
            .items
            .iter()
            .flat_map(|(base, item)| item.stmts.iter().map(move |stmt| stmt.span.shift(*base)))
            .filter(|span| reads_inside(&src[span_range(*span)]))
            .map(|span| span.end)
            .max();
        let mut hidden = Vec::new();
        let mut stmts: Vec<(u32, &Stmt, bool)> = Vec::new();
        for (base, item) in &parsed.items {
            // A `@multitest fn` reads one case: its body is read like
            // the lines below an `@!multitest` divider.
            let item_stmts = item.stmts.iter().flat_map(|stmt| {
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
                std::iter::once(stmt).chain(body)
            });
            let mut expanded = Vec::new();
            for stmt in item_stmts {
                expand(*base, stmt, &readers, 0, &mut expanded);
            }
            stmts.extend(expanded.into_iter().map(|(base, stmt)| (base, stmt, false)));
        }
        // The interactor is read once, after every top-level item (so
        // `@!multitest`, constants and types in later items are seen first).
        if let Some((judge_base, block)) = judge {
            stmts.extend(block.stmts.iter().map(|stmt| (judge_base, stmt, true)));
        }
        for (base, stmt, in_judge) in stmts {
            // A reader's lines, like the interactor's, are offset from
            // its own item.
            builder.base = base;
            // Decorated fns (`@brute`, `@check`, `@interactor`)
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
                StmtKind::TypeAlias {
                    name, generics, ty, ..
                } if generics.is_empty() => {
                    builder
                        .aliases
                        .insert(name.text.clone(), (builder.base, ty.clone()));
                }
                StmtKind::Use {
                    path, names, glob, ..
                } if path.first().is_some_and(|root| root.text == "std") => {
                    let module: Vec<String> = path.iter().map(|n| n.text.clone()).collect();
                    let names: Vec<&str> = names.iter().map(|n| n.text.as_str()).collect();
                    for (name, parts) in std_structs(&module, &names, *glob) {
                        builder
                            .records
                            .entry(name)
                            .or_insert((0, Record::Shaped(parts)));
                    }
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
                    init: Some(init), ..
                } if !in_judge
                    && judge.is_none()
                    && above.is_none()
                    && !multitest
                    && read_gen(init).is_some() =>
                {
                    let func = read_gen(init).expect("tested above");
                    let line = lines.line_of(stmt.span.shift(builder.base).start) + 1;
                    let program = gen_program(src, &func, line).ok_or_else(|| {
                            builder.error(stmt.span, format!("`cpl stress` can't run `{func}` on its own to make this input; check that the file builds"))
                        })?;
                    builder.spec.fields.push(Field {
                        names: vec![None],
                        shape: Shape::Gen {
                            func,
                            program: std::sync::Arc::new(program),
                        },
                        line,
                    });
                }
                StmtKind::Let {
                    pat,
                    ty,
                    init: Some(init),
                    ..
                } if typed_read(ty.as_ref(), init).is_some() || read_none(init).is_some() => {
                    let Some((ty, none)) = typed_read(ty.as_ref(), init) else {
                        return Err(builder.error(
                                stmt.span,
                                "this `read()` has no type, so there is nothing to generate from; write `let n: 1..=100 = read()`".into(),
                            ));
                    };
                    let shape = builder.shape_written(&ty, none)?;
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
                        index: None,
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
                StmtKind::For { .. } if (in_judge || judge.is_none()) && builder.reads(stmt) => {
                    let Some(shape) = builder.pass(stmt, &HashSet::new())? else {
                        if let Some(shapes) = builder.inlined(stmt)? {
                            let line = lines.line_of(stmt.span.shift(builder.base).start) + 1;
                            let fields = shapes.into_iter().map(|shape| Field {
                                names: vec![None],
                                shape,
                                line,
                            });
                            match in_judge {
                                true => hidden.extend(fields),
                                false => builder.spec.fields.extend(fields),
                            }
                            continue;
                        }
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
                // describes; `@brute`, `@check` and the like read for
                // themselves.
                StmtKind::Fn(decl)
                    if (in_judge || judge.is_none())
                        && !readers.contains_key(decl.name.text.as_str())
                        && !builder.values.contains_key(&decl.name.text)
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
                            "`cpl stress` makes a `read(gen: ..)` or `read(none: ..)` only as a top-level `let name: Type = read(..)` outside a test case, so a generated input would leave this one out; write a `@gen fn` that prints a whole input, or pass `--gen <program>`".into(),
                        ));
                }
                _ if (in_judge || judge.is_none()) && builder.reads(stmt) => {
                    if let StmtKind::Expr(expr) = &stmt.kind {
                        let last = Some(stmt.span.shift(builder.base).end) == last_read;
                        if !in_judge && last && readers.is_empty() && builder.top_choice(expr)? {
                            continue;
                        }
                    }
                    if let Some(shapes) = builder.inlined(stmt)? {
                        let line = lines.line_of(stmt.span.shift(builder.base).start) + 1;
                        let fields = shapes.into_iter().map(|shape| Field {
                            names: vec![None],
                            shape,
                            line,
                        });
                        match in_judge {
                            true => hidden.extend(fields),
                            false => builder.spec.fields.extend(fields),
                        }
                        continue;
                    }
                    return Err(builder.error(
                            stmt.span,
                            "this `read()` is not a top-level `let name: Type = read()`, so a generated input would leave it out; read it into one first, like `let edges: [m](u: V, v: V) = read()`".into(),
                        ));
                }
                _ => {}
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

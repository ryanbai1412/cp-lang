//! The largest and smallest value an expression can take, from the input
//! bounds the program declares.
//!
//! `let n: 1..=2e5 = read()` says `n` is at most 200000, `const K = 20`
//! says `K` is 20, and `domain V = 1..=n` then has at most 200000 values.
//! Anything this cannot see through is `None`, and the caller stays quiet.

use crate::File;
use cpl_syntax::ast::{BinOp, Expr, ExprKind, PatternKind, StmtKind, Type, TypeKind, UnOp};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// An inclusive range of integers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Interval {
    pub lo: i128,
    pub hi: i128,
}

impl Interval {
    pub(crate) fn hull(self, other: Interval) -> Interval {
        Interval {
            lo: self.lo.min(other.lo),
            hi: self.hi.max(other.hi),
        }
    }

    /// How many integers it holds; 0 when it is empty.
    pub(crate) fn count(self) -> u128 {
        if self.hi < self.lo {
            0
        } else {
            (self.hi - self.lo) as u128 + 1
        }
    }
}

/// File-wide tables computed once in [`Bounds::of`] and shared across scope clones.
#[derive(Default, Clone)]
struct FileFacts {
    /// The number of values of each domain, at most.
    domains: HashMap<String, u128>,
    aliases: HashMap<String, Type>,
    /// How many places in the file bind each value name.
    bindings: HashMap<String, u32>,
    /// Names a fn or lambda body assigns, which a call anywhere can change.
    fn_writes: HashSet<String>,
    /// Sequences bound once whose length only statements of their own scope
    /// change, and `g[]` for the rows of such a `g`: their `.len` is a name
    /// the relations know until `assigned_in` finds a change. A change in a
    /// fn or lambda body declared outside the sequence could come from any
    /// call, so it gets no entry.
    tracked: HashSet<String>,
    /// Names some binding or assignment of reads a `.len`: their intervals
    /// reach `LEN_MAX`, which says nothing worth showing.
    long: HashSet<String>,
    /// Lists that start empty and only `push` or `pop` change, all in their
    /// own scope: how many `push` calls the source has for each.
    pushed: HashMap<String, usize>,
}

/// What the top level of a file says about its names.
#[derive(Default, Clone)]
pub(crate) struct Bounds {
    file: Rc<FileFacts>,
    values: HashMap<String, Interval>,
    /// Fns the program declares, which hide the prelude's `min`, `max`, `abs`.
    fns: Rc<HashSet<String>>,
    /// How each name the prover can reason about got its value.
    defs: HashMap<String, Def>,
    /// What a condition proves about `a - b` for two bounded names, which
    /// their own intervals cannot say (`lo < hi` makes `hi - lo` positive).
    diffs: HashMap<(String, String), Interval>,
    /// The value a `var` holds where nothing has assigned it since it was
    /// declared or last set, which its hull over the whole scope cannot say.
    now: HashMap<String, Interval>,
    /// Arrays bound with a type that bounds their cells: how many at most,
    /// as a number and as factors of the source.
    sizes: HashMap<String, (u128, Vec<super::Factor>)>,
}

/// The most a `.len` is taken to be: only relations bound it usefully.
pub(crate) const LEN_MAX: i128 = i64::MAX as i128;

/// A name the prover can be told about: an input of a declared range, or an
/// immutable `let` of arithmetic over such names, as prover text.
#[derive(Clone)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
enum Def {
    Input(Interval),
    Let { text: String, names: Vec<String> },
}

/// `expr` as fully parenthesized prover text and the names it reads, when
/// it is only integers, names and arithmetic.
fn prover_text(expr: &Expr, names: &mut Vec<String>) -> Option<String> {
    Some(match &expr.kind {
        ExprKind::Int(value) => value.to_string(),
        ExprKind::Name(name) => {
            names.push(name.clone());
            name.clone()
        }
        ExprKind::Paren(inner) => prover_text(inner, names)?,
        ExprKind::Unary {
            op: UnOp::Neg,
            expr,
        } => format!("(-{})", prover_text(expr, names)?),
        ExprKind::Binary { op, lhs, rhs, .. } => {
            let op = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Rem => "%",
                BinOp::Pow => "**",
                _ => return None,
            };
            format!("({} {op} {})", prover_text(lhs, names)?, prover_text(rhs, names)?)
        }
        _ => return None,
    })
}

impl Bounds {
    pub(crate) fn of(file: &File) -> Bounds {
        let mut bounds = Bounds::default();
        bounds.count_bindings(file);
        // In source order: a bound can only use what is declared above it.
        for (base, stmt) in file.top_level() {
            match &stmt.kind {
                StmtKind::Const {
                    name,
                    init: Some(init),
                    ..
                } => {
                    if let Some(value) = bounds.expr(init) {
                        bounds.values.insert(name.text.clone(), value);
                    }
                }
                StmtKind::Let {
                    pat, ty: Some(ty), ..
                } => bounds.declare(base, &super::binding(pat), ty),
                StmtKind::Let {
                    pat,
                    ty: None,
                    init: Some(init),
                    ..
                } if read_type(init).is_some() => bounds.declare(base, &super::binding(pat), read_type(init).unwrap()),
                StmtKind::Let {
                    mutable: false,
                    pat,
                    ty: None,
                    init: Some(init),
                    ..
                } => {
                    if let (PatternKind::Name(name), Some(value)) = (&super::binding(pat).kind, bounds.expr(init)) {
                        bounds.values.insert(name.clone(), value);
                        bounds.define(name, init);
                    }
                }
                StmtKind::Domain { name, def } => {
                    let size = match &def.kind {
                        ExprKind::Range {
                            lo: Some(lo),
                            hi: Some(hi),
                            inclusive,
                            step: None,
                        } => bounds.span(lo, hi, *inclusive).map(Interval::count),
                        _ => None,
                    };
                    if let Some(size) = size {
                        Rc::make_mut(&mut bounds.file)
                            .domains
                            .insert(name.text.clone(), size);
                    }
                }
                StmtKind::Fn(decl) => bounds.declare_fn(&decl.name.text),
                StmtKind::Use {
                    path,
                    names,
                    glob,
                    alias,
                } => {
                    if *glob {
                        for name in ["min", "max", "abs"] {
                            bounds.declare_fn(name);
                        }
                    }
                    for name in path.last().into_iter().chain(names).chain(alias) {
                        bounds.declare_fn(&name.text);
                    }
                }
                StmtKind::TypeAlias { name, generics, ty } if generics.is_empty() => {
                    Rc::make_mut(&mut bounds.file)
                        .aliases
                        .insert(name.text.clone(), ty.clone());
                }
                _ => {}
            }
        }
        bounds
    }

    /// `pat` is bound to a value of type `ty`, part by part for a tuple.
    fn declare(&mut self, base: u32, pat: &cpl_syntax::ast::Pattern, ty: &Type) {
        match (&pat.kind, &ty.kind) {
            (PatternKind::Name(name), _) => {
                if let Some(range) = self.range(ty) {
                    self.input(name, range);
                }
                if let Some(size) = super::array_size(base, ty, self) {
                    self.set_size(name, size);
                }
            }
            (PatternKind::Tuple(pats), TypeKind::Tuple(tys)) if pats.len() == tys.len() => {
                for (pat, ty) in pats.iter().zip(tys) {
                    self.declare(base, pat, ty);
                }
            }
            _ => {}
        }
    }

    /// From here on `name` is in `value`, or unknown: a local that
    /// shadows a bounded name hides its bound.
    fn count_bindings(&mut self, file: &File) {
        use cpl_syntax::ast::{Pattern, Stmt};
        use cpl_syntax::visit::{walk_expr, walk_stmt, Visitor};
        struct Count<'a>(&'a mut HashMap<String, u32>, &'a mut HashSet<String>);
        struct Writes<'a>(&'a mut HashSet<(String, u32)>);
        struct Decls(Vec<(String, u32)>);
        impl Visitor for Decls {
            fn pattern(&mut self, pat: &Pattern) {
                match &pat.kind {
                    PatternKind::Name(n) => self.0.push((n.clone(), pat.span.start)),
                    PatternKind::Domain(n) => self.0.push((n.text.clone(), pat.span.start)),
                    PatternKind::Tuple(parts) | PatternKind::Or(parts) => parts.iter().for_each(|p| self.pattern(p)),
                    PatternKind::Variant { args, .. } => args.iter().for_each(|p| self.pattern(p)),
                    PatternKind::Var(inner) => self.pattern(inner),
                    PatternKind::Wildcard | PatternKind::Literal(_) | PatternKind::Error => {}
                }
            }
        }
        fn written(e: &Expr, out: &mut HashSet<(String, u32)>) {
            match &e.kind {
                ExprKind::Name(n) => {
                    out.insert((n.clone(), e.span.start));
                }
                ExprKind::Paren(inner) => written(inner, out),
                ExprKind::Tuple(items) | ExprKind::List(items) => items.iter().for_each(|i| written(i, out)),
                _ => {}
            }
        }
        impl Visitor for Writes<'_> {
            fn stmt(&mut self, stmt: &Stmt) {
                match &stmt.kind {
                    StmtKind::Assign { target, .. } => written(target, self.0),
                    StmtKind::Lend { place, .. } => written(place, self.0),
                    _ => {}
                }
            }
            fn expr(&mut self, expr: &Expr) {
                if let ExprKind::Call { args, .. } = &expr.kind {
                    for a in args.iter().filter(|a| a.mutable) {
                        written(&a.value, self.0);
                    }
                }
            }
        }
        impl Count<'_> {
            fn add(&mut self, name: &str) {
                *self.0.entry(name.to_string()).or_default() += 1;
            }
            fn names(&mut self, pat: &Pattern) {
                match &pat.kind {
                    PatternKind::Name(n) => self.add(n),
                    PatternKind::Domain(n) => self.add(&n.text),
                    PatternKind::Tuple(parts) | PatternKind::Or(parts) => {
                        parts.iter().for_each(|p| self.names(p))
                    }
                    PatternKind::Variant { args, .. } => args.iter().for_each(|p| self.names(p)),
                    PatternKind::Var(inner) => self.names(inner),
                    PatternKind::Wildcard | PatternKind::Literal(_) | PatternKind::Error => {}
                }
            }
        }
        impl Visitor for Count<'_> {
            fn stmt(&mut self, stmt: &Stmt) {
                match &stmt.kind {
                    StmtKind::Fn(decl) => {
                        decl.params.iter().for_each(|p| self.add(&p.name.text));
                        // A write to a name this fn declared before it is to
                        // the fn's own local, which no call can see.
                        let mut writes = HashSet::new();
                        walk_stmt(stmt, &mut Writes(&mut writes));
                        let mut decls = Decls(decl.params.iter().map(|p| (p.name.text.clone(), 0)).collect());
                        walk_stmt(stmt, &mut decls);
                        let local = |(n, at): &(String, u32)| decls.0.iter().any(|(d, from)| d == n && from < at);
                        self.1.extend(writes.iter().filter(|w| !local(w)).map(|(n, _)| n.clone()));
                    }
                    StmtKind::Const { name, .. } => self.add(&name.text),
                    StmtKind::Let { pat, .. } if matches!(pat.kind, PatternKind::Variant { .. }) => {
                        if let PatternKind::Name(n) = &super::binding(pat).kind {
                            self.add(n);
                        }
                    }
                    _ => {}
                }
            }
            fn expr(&mut self, expr: &Expr) {
                if let ExprKind::Lambda { body, .. } = &expr.kind {
                    let mut writes = HashSet::new();
                    walk_expr(body, &mut Writes(&mut writes));
                    self.1.extend(writes.into_iter().map(|(n, _)| n));
                }
            }
            fn pattern(&mut self, pat: &Pattern) {
                self.names(pat);
            }
        }
        {
            let file_facts = Rc::make_mut(&mut self.file);
            let mut count = Count(&mut file_facts.bindings, &mut file_facts.fn_writes);
            file.top_level().for_each(|(_, s)| walk_stmt(s, &mut count));
        }
        self.find_fixed(file);
    }

    fn find_fixed(&mut self, file: &File) {
        use cpl_syntax::ast::{Pattern, Stmt};
        use cpl_syntax::visit::{walk_stmt, Visitor};
        #[derive(Default)]
        struct Scan {
            base: u32,
            changed: HashSet<String>,
            changes: Vec<(String, u32, u32)>,
            bodies: Vec<(u32, u32)>,
            decls: HashMap<String, u32>,
            /// `(target, directly_reads_len, referenced_names)` for each binding/assignment.
            sources: Vec<(String, bool, Vec<String>)>,
            has_len: bool,
            seqs: HashSet<String>,
            empty: HashSet<String>,
            pushes: HashMap<String, usize>,
            pops: HashMap<String, usize>,
        }
        impl Visitor for Scan {
            fn stmt(&mut self, stmt: &Stmt) {
                let at = (self.base + stmt.span.start, self.base + stmt.span.end);
                if let StmtKind::Fn(decl) = &stmt.kind {
                    self.bodies.push(at);
                    for p in &decl.params {
                        self.decls.insert(p.name.text.clone(), at.0);
                    }
                }
                match &stmt.kind {
                    StmtKind::Assign { target, value, .. } => {
                        if let Some(n) = root(target) {
                            self.change(n, at);
                            self.record_source(n.to_string(), value);
                        }
                        self.change_row(target, at);
                    }
                    StmtKind::Lend { place, .. } => {
                        if let Some(n) = root(place) {
                            self.change(n, at);
                        }
                        self.change_row(place, at);
                    }
                    _ => {}
                }
                if let StmtKind::Let { pat, init, .. } = &stmt.kind {
                    let mut names = vec![pat];
                    while let Some(p) = names.pop() {
                        match &p.kind {
                            PatternKind::Name(n) => {
                                self.decls.insert(n.clone(), self.base + p.span.start);
                            }
                            PatternKind::Tuple(ps) => names.extend(ps),
                            _ => {}
                        }
                    }
                    if let Some(init) = init {
                        if let PatternKind::Name(n) = &pat.kind {
                            self.record_source(n.clone(), init);
                            if matches!(&init.kind, ExprKind::List(items) if items.is_empty()) {
                                self.empty.insert(n.clone());
                            }
                        }
                        if let (PatternKind::Tuple(ps), ExprKind::Tuple(es)) = (&pat.kind, &init.kind) {
                            for (p, e) in ps.iter().zip(es) {
                                if let PatternKind::Name(n) = &p.kind {
                                    self.record_source(n.clone(), e);
                                }
                            }
                        }
                    }
                }
            }
            fn expr(&mut self, expr: &Expr) {
                let at = (self.base + expr.span.start, self.base + expr.span.end);
                match &expr.kind {
                    ExprKind::Lambda { .. } => self.bodies.push(at),
                    ExprKind::Call { callee, args } => {
                        if let ExprKind::Field { base, name } = &callee.kind {
                            if !keeps_len(&name.text) {
                                if let Some(n) = root(base) {
                                    self.change(n, at);
                                    if let ExprKind::Name(n) = &base.kind {
                                        match (name.text.as_str(), args.len()) {
                                            ("push", 1) => *self.pushes.entry(n.clone()).or_default() += 1,
                                            ("pop", 0) => *self.pops.entry(n.clone()).or_default() += 1,
                                            _ => {}
                                        }
                                    }
                                }
                                self.change_row(base, at);
                            }
                        }
                        for a in args.iter().filter(|a| a.mutable) {
                            if let Some(n) = root(&a.value) {
                                self.change(n, at);
                            }
                            self.change_row(&a.value, at);
                        }
                    }
                    ExprKind::Field { base, name } if name.text == "len" => {
                        if let Some(n) = root(base) {
                            self.seqs.insert(n.to_string());
                        }
                        if let ExprKind::Index { base, indices } = &base.kind {
                            if let (Some(n), 1) = (root(base), indices.len()) {
                                self.seqs.insert(format!("{n}[]"));
                            }
                        }
                    }
                    _ => {}
                }
            }
            fn pattern(&mut self, _: &Pattern) {}
        }
        impl Scan {
            fn record_source(&mut self, target: String, expr: &Expr) {
                struct Refs {
                    has_len: bool,
                    names: Vec<String>,
                }
                impl Visitor for Refs {
                    fn expr(&mut self, e: &Expr) {
                        match &e.kind {
                            ExprKind::Name(n) => self.names.push(n.clone()),
                            ExprKind::Field { name, .. } if name.text == "len" => {
                                self.has_len = true;
                            }
                            _ => {}
                        }
                    }
                }
                let mut refs = Refs {
                    has_len: false,
                    names: Vec::new(),
                };
                cpl_syntax::visit::walk_expr(expr, &mut refs);
                self.has_len |= refs.has_len;
                self.sources.push((target, refs.has_len, refs.names));
            }
            fn change(&mut self, name: &str, at: (u32, u32)) {
                self.changed.insert(name.to_string());
                self.changes.push((name.to_string(), at.0, at.1));
            }
            /// `g[]` stands for the length of every row of `g`.
            fn change_row(&mut self, place: &Expr, at: (u32, u32)) {
                if let Some(n) = row_root(place) {
                    self.change(&format!("{n}[]"), at);
                }
            }
            /// Whether only statements of `name`'s own scope change its length.
            fn local(&self, name: &str) -> bool {
                let mut starts = self.changes.iter().filter(|(n, ..)| n == name).map(|c| c.1).peekable();
                if starts.peek().is_none() {
                    return true;
                }
                let Some(&decl) = self.decls.get(name.strip_suffix("[]").unwrap_or(name)) else {
                    return false;
                };
                let outside = |&(s, e): &(u32, u32), at: u32| s <= at && at < e && !(s <= decl && decl < e);
                !starts.any(|at| self.bodies.iter().any(|b| outside(b, at)))
            }
        }
        let mut scan = Scan::default();
        for (base, stmt) in file.top_level() {
            scan.base = base;
            walk_stmt(stmt, &mut scan);
        }
        let tracked: HashSet<String> = scan
            .seqs
            .iter()
            .filter(|n| {
                let seq = n.strip_suffix("[]").unwrap_or(n);
                self.bound_once(seq) && !self.file.fn_writes.contains(seq) && scan.local(n)
            })
            .cloned()
            .collect();
        let pushed: HashMap<String, usize> = scan
            .empty
            .iter()
            .filter(|n| self.bound_once(n) && !self.file.fn_writes.contains(*n) && scan.local(n))
            .filter_map(|n| {
                let pushes = scan.pushes.get(n).copied().unwrap_or(0);
                let changes = scan.changes.iter().filter(|c| c.0 == *n).count();
                (changes == pushes + scan.pops.get(n).copied().unwrap_or(0)).then(|| (n.clone(), pushes))
            })
            .collect();
        {
            let file_facts = Rc::make_mut(&mut self.file);
            file_facts.tracked = tracked;
            file_facts.pushed = pushed;
        }
        if scan.has_len {
            loop {
                let more: Vec<String> = scan
                    .sources
                    .iter()
                    .filter(|(n, direct, refs)| {
                        !self.file.long.contains(n)
                            && (*direct || refs.iter().any(|r| self.file.long.contains(r)))
                    })
                    .map(|(n, ..)| n.clone())
                    .collect();
                if more.is_empty() {
                    break;
                }
                Rc::make_mut(&mut self.file).long.extend(more);
            }
        }
    }

    /// `xs.len` for a sequence whose length is fixed, as the name relations use.
    pub(crate) fn atom(&self, e: &Expr) -> Option<String> {
        match &e.kind {
            ExprKind::Name(n) => Some(n.clone()),
            ExprKind::Paren(inner) => self.atom(inner),
            ExprKind::Field { base, name } if name.text == "len" => match &base.kind {
                ExprKind::Name(n) if self.file.tracked.contains(n) => {
                    Some(format!("{n}.len"))
                }
                ExprKind::Index { base, indices } => match (&base.kind, &indices[..]) {
                    (ExprKind::Name(g), [row]) => {
                        match &row.kind {
                            ExprKind::Name(u) if self.file.tracked.contains(&format!("{g}[]")) => {
                                Some(format!("{g}[{u}].len"))
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }

    /// Whether `e` reads a `.len` or a name whose value comes from one.
    pub(crate) fn reads_len(&self, e: &Expr) -> bool {
        use cpl_syntax::visit::{walk_expr, Visitor};
        struct Reads<'a>(&'a Bounds, bool);
        impl Visitor for Reads<'_> {
            fn expr(&mut self, e: &Expr) {
                self.1 |= match &e.kind {
                    ExprKind::Name(n) => self.0.file.long.contains(n),
                    ExprKind::Field { name, .. } => name.text == "len",
                    _ => false,
                };
            }
        }
        let mut reads = Reads(self, false);
        walk_expr(e, &mut reads);
        reads.1
    }

    pub(crate) fn is_long(&self, name: &str) -> bool {
        self.file.long.contains(name)
    }

    /// Whether one place in the file binds `name`, so every use of it
    /// that resolves at all reads that binding.
    pub(crate) fn written_by_fn(&self, name: &str) -> bool {
        let seq = name.strip_suffix(".len").unwrap_or(name);
        match row_of(name) {
            Some((g, u)) => self.file.fn_writes.contains(g) || self.file.fn_writes.contains(u),
            None => self.file.fn_writes.contains(seq),
        }
    }

    pub(crate) fn bound_once(&self, name: &str) -> bool {
        self.file.bindings.get(name) == Some(&1)
    }

    pub(crate) fn bind(&mut self, name: &str, value: Option<Interval>) {
        self.defs.remove(name);
        self.sizes.remove(name);
        self.unrelate(name);
        match value {
            Some(value) => self.values.insert(name.to_string(), value),
            None => self.values.remove(name),
        };
    }

    /// From here on `name` is an array of at most `size` cells.
    pub(crate) fn set_size(&mut self, name: &str, size: (u128, Vec<super::Factor>)) {
        self.sizes.insert(name.to_string(), size);
    }

    pub(crate) fn size(&self, name: &str) -> Option<&(u128, Vec<super::Factor>)> {
        self.sizes.get(name)
    }

    /// How many `push` calls build `name`, a list nothing else lengthens.
    pub(crate) fn pushes(&self, name: &str) -> Option<usize> {
        self.file.pushed.get(name).copied()
    }

    /// A test proved `name` is in `value`: what was known about it stays.
    pub(crate) fn restrict(&mut self, name: &str, value: Interval) {
        self.defs.remove(name);
        self.values.insert(name.to_string(), value);
    }

    /// From here on `name` is an input anywhere in `range`.
    pub(crate) fn input(&mut self, name: &str, range: Interval) {
        self.bind(name, Some(range));
        self.defs.insert(name.to_string(), Def::Input(range));
    }

    /// `name` was just bound to the value of `init`, which the prover can
    /// follow when it is arithmetic over names it can follow.
    pub(crate) fn define(&mut self, name: &str, init: &Expr) {
        let mut names = Vec::new();
        if let Some(text) = prover_text(init, &mut names) {
            if names.iter().all(|n| self.defs.contains_key(n)) {
                self.defs.insert(name.to_string(), Def::Let { text, names });
            }
        }
    }

    /// Inputs in their declared ranges for which `expr` itself leaves
    /// `Int`, found and checked by the prover, as `n = 55110`.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn overflow(&self, expr: &Expr) -> Option<String> {
        fn tell(
            bounds: &Bounds,
            name: &str,
            ctx: &mut cpl_prover::Context,
            told: &mut HashSet<String>,
        ) -> Option<()> {
            if !told.insert(name.to_string()) {
                return Some(());
            }
            match bounds.defs.get(name)? {
                Def::Input(range) => ctx.var(name, &format!("{}..={}", range.lo, range.hi)).ok(),
                Def::Let { text, names } => {
                    for n in names {
                        tell(bounds, n, ctx, told)?;
                    }
                    ctx.define(name, text).ok()
                }
            }
        }
        let mut names = Vec::new();
        let text = prover_text(expr, &mut names)?;
        let mut ctx = cpl_prover::Context::new();
        let mut told = HashSet::new();
        for name in &names {
            tell(self, name, &mut ctx, &mut told)?;
        }
        let whole = cpl_prover::parse::term(&text).ok()?.to_string();
        let findings = ctx.check_overflow(&text).ok()?;
        let finding = findings.into_iter().find(|f| f.expr == whole)?;
        finding.verdict.counterexample().map(|c| c.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn overflow(&self, _expr: &Expr) -> Option<String> {
        None
    }

    /// From here on `a - b` is in `diff`, as well as in what the intervals
    /// of `a` and `b` give.
    pub(crate) fn relate(&mut self, a: &str, b: &str, diff: Interval) {
        let key = (a.to_string(), b.to_string());
        let diff = match self.diffs.get(&key) {
            Some(had) => Interval {
                lo: had.lo.max(diff.lo),
                hi: had.hi.min(diff.hi),
            },
            None => diff,
        };
        self.diffs.insert(key, diff);
    }

    /// The `var` `name` holds a value in `value` until it is next assigned.
    pub(crate) fn set_now(&mut self, name: &str, value: Interval) {
        if !self.written_by_fn(name) {
            self.now.insert(name.to_string(), value);
        }
    }

    /// Every name an assignment can make a relation or `now` value stale for.
    pub(crate) fn related(&self) -> Vec<String> {
        let pairs = self.diffs.keys().flat_map(|(a, b)| [a.clone(), b.clone()]);
        let mut names: Vec<String> = pairs.chain(self.now.keys().cloned()).collect();
        names.sort();
        names.dedup();
        names
    }

    /// `name` was assigned: what a condition proved about it may not hold.
    pub(crate) fn unrelate(&mut self, name: &str) {
        let gone = |n: &str| n == name || row_of(n).is_some_and(|(_, u)| u == name);
        self.diffs.retain(|(a, b), _| !gone(a) && !gone(b));
        self.now.remove(name);
    }

    /// Every relation, as `(a, b, what a - b is in)`.
    pub(crate) fn relations(&self) -> Vec<(String, String, Interval)> {
        self.diffs.iter().map(|((a, b), d)| (a.clone(), b.clone(), *d)).collect()
    }

    /// Every `var` with a `now` value.
    pub(crate) fn nows(&self) -> Vec<(String, Interval)> {
        self.now.iter().map(|(n, v)| (n.clone(), *v)).collect()
    }

    pub(crate) fn now_of(&self, name: &str) -> Option<Interval> {
        self.now.get(name).copied()
    }

    /// What the relations between two names prove about `a - b`.
    pub(crate) fn diff(&self, a: &Expr, b: &Expr) -> Option<Interval> {
        self.name_diff(&self.atom(a)?, &self.atom(b)?)
    }

    pub(crate) fn name_diff(&self, a: &str, b: &str) -> Option<Interval> {
        if let Some(d) = self.diffs.get(&(a.to_string(), b.to_string())) {
            return Some(*d);
        }
        let d = self.diffs.get(&(b.to_string(), a.to_string()))?;
        Some(Interval {
            lo: d.hi.checked_neg()?,
            hi: d.lo.checked_neg()?,
        })
    }

    /// Everything known here about `a - b`: their relation, and the values
    /// they hold now or else anywhere in scope.
    pub(crate) fn current_diff(&self, a: &str, b: &str) -> Option<Interval> {
        let d = self.value_diff(a, b)?;
        let d = match self.name_diff(a, b) {
            Some(r) => Interval { lo: d.lo.max(r.lo), hi: d.hi.min(r.hi) },
            None => d,
        };
        (d.lo <= d.hi).then_some(d)
    }

    /// What the names' values alone allow `a - b` to be.
    pub(crate) fn value_diff(&self, a: &str, b: &str) -> Option<Interval> {
        let value = |n: &str| self.now.get(n).copied().or_else(|| self.expr_name(n));
        Self::combine(BinOp::Sub, value(a)?, value(b)?)
    }

    /// The interval a bounded name is in.
    pub(crate) fn expr_name(&self, name: &str) -> Option<Interval> {
        match name.strip_suffix(".len") {
            Some(seq) if self.file.tracked.contains(seq) => Some(Interval { lo: 0, hi: LEN_MAX }),
            Some(_) if row_of(name).is_some_and(|(g, _)| self.file.tracked.contains(&format!("{g}[]"))) => {
                Some(Interval { lo: 0, hi: LEN_MAX })
            }
            _ => self.values.get(name).copied(),
        }
    }

    /// The values `lo..hi` or `lo..=hi` can cover.
    pub(crate) fn span(&self, lo: &Expr, hi: &Expr, inclusive: bool) -> Option<Interval> {
        let (lo, hi) = (self.expr(lo)?, self.expr(hi)?);
        Some(Interval {
            lo: lo.lo,
            hi: if inclusive {
                hi.hi
            } else {
                hi.hi.checked_sub(1)?
            },
        })
    }

    pub(crate) fn range(&self, ty: &Type) -> Option<Interval> {
        match &ty.kind {
            TypeKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
            } => self.span(lo, hi, *inclusive),
            _ => None,
        }
    }

    /// The interval an expression's value lies in.
    pub(crate) fn expr(&self, expr: &Expr) -> Option<Interval> {
        match &expr.kind {
            ExprKind::Int(value) => Some(Interval {
                lo: *value,
                hi: *value,
            }),
            ExprKind::Name(name) => self.values.get(name).copied(),
            ExprKind::Paren(inner) => self.expr(inner),
            ExprKind::Field { .. } => self.atom(expr).and_then(|a| self.expr_name(&a)),
            ExprKind::Unary {
                op: UnOp::Neg,
                expr,
            } => {
                let inner = self.expr(expr)?;
                Some(Interval {
                    lo: inner.hi.checked_neg()?,
                    hi: inner.lo.checked_neg()?,
                })
            }
            ExprKind::Binary { op, lhs, rhs, .. } => {
                let value = Self::combine(*op, self.expr(lhs)?, self.expr(rhs)?)?;
                match self.diff(lhs, rhs).filter(|_| *op == BinOp::Sub) {
                    Some(d) => Some(Interval {
                        lo: value.lo.max(d.lo),
                        hi: value.hi.min(d.hi),
                    })
                    .filter(|v| v.lo <= v.hi),
                    None => Some(value),
                }
            }
            ExprKind::Call { callee, args } => {
                let ExprKind::Name(name) = &callee.kind else {
                    return None;
                };
                if self.fns.contains(name) || args.iter().any(|a| a.name.is_some() || a.mutable) {
                    return None;
                }
                let values = args
                    .iter()
                    .map(|a| self.expr(&a.value))
                    .collect::<Option<Vec<_>>>()?;
                match (name.as_str(), values.as_slice()) {
                    ("abs", [x]) => Some(if x.lo >= 0 {
                        *x
                    } else if x.hi <= 0 {
                        Interval {
                            lo: x.hi.checked_neg()?,
                            hi: x.lo.checked_neg()?,
                        }
                    } else {
                        Interval {
                            lo: 0,
                            hi: x.lo.checked_neg()?.max(x.hi),
                        }
                    }),
                    ("max", [first, rest @ ..]) if !rest.is_empty() => {
                        Some(rest.iter().fold(*first, |a, b| Interval {
                            lo: a.lo.max(b.lo),
                            hi: a.hi.max(b.hi),
                        }))
                    }
                    ("min", [first, rest @ ..]) if !rest.is_empty() => {
                        Some(rest.iter().fold(*first, |a, b| Interval {
                            lo: a.lo.min(b.lo),
                            hi: a.hi.min(b.hi),
                        }))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// The values `a op b` can take.
    pub(crate) fn combine(op: BinOp, a: Interval, b: Interval) -> Option<Interval> {
        match op {
            BinOp::Add => Some(Interval {
                lo: a.lo.checked_add(b.lo)?,
                hi: a.hi.checked_add(b.hi)?,
            }),
            BinOp::Sub => Some(Interval {
                lo: a.lo.checked_sub(b.hi)?,
                hi: a.hi.checked_sub(b.lo)?,
            }),
            BinOp::Mul => {
                let corners = [
                    a.lo.checked_mul(b.lo)?,
                    a.lo.checked_mul(b.hi)?,
                    a.hi.checked_mul(b.lo)?,
                    a.hi.checked_mul(b.hi)?,
                ];
                Some(Interval {
                    lo: *corners.iter().min()?,
                    hi: *corners.iter().max()?,
                })
            }
            BinOp::Div if b.lo > 0 => {
                let corners = [
                    a.lo.checked_div_euclid(b.lo)?,
                    a.lo.checked_div_euclid(b.hi)?,
                    a.hi.checked_div_euclid(b.lo)?,
                    a.hi.checked_div_euclid(b.hi)?,
                ];
                Some(Interval {
                    lo: *corners.iter().min()?,
                    hi: *corners.iter().max()?,
                })
            }
            BinOp::Rem if b.lo > 0 => Some(if a.lo >= 0 && a.hi < b.lo {
                a
            } else {
                let hi = b.hi - 1;
                Interval {
                    lo: 0,
                    hi: if a.lo >= 0 { hi.min(a.hi) } else { hi },
                }
            }),
            BinOp::Pow if b.lo == b.hi && b.lo >= 0 => {
                let k = u32::try_from(b.lo).ok()?;
                let (lo, hi) = (a.lo.checked_pow(k)?, a.hi.checked_pow(k)?);
                Some(if k % 2 == 1 || a.lo >= 0 {
                    Interval { lo, hi }
                } else if a.hi <= 0 {
                    Interval { lo: hi, hi: lo }
                } else {
                    Interval { lo: 0, hi: lo.max(hi) }
                })
            }
            _ => None,
        }
    }

    /// From here on `name` is a fn of the program's own, not the prelude's.
    pub(crate) fn declare_fn(&mut self, name: &str) {
        if !self.fns.contains(name) {
            Rc::make_mut(&mut self.fns).insert(name.to_string());
        }
    }

    /// How many values the domain `name` has, at most.
    pub(crate) fn domains(&self) -> Vec<(String, u128)> {
        let mut out: Vec<(String, u128)> = self.file.domains.iter().map(|(d, n)| (d.clone(), *n)).collect();
        out.sort();
        out
    }

    pub(crate) fn domain_size(&self, name: &str) -> Option<u128> {
        self.file.domains.get(name).copied()
    }

    /// How many values a type has, at most.
    pub(crate) fn type_size(&self, ty: &Type) -> Option<u128> {
        match &ty.kind {
            TypeKind::Range { .. } => self.range(ty).map(Interval::count),
            TypeKind::Named { name, args } if args.is_empty() => match name.text.as_str() {
                "Bool" => Some(2),
                "Char" | "U8" => Some(256),
                other => self
                    .file
                    .domains
                    .get(other)
                    .copied()
                    .or_else(|| self.type_size(self.file.aliases.get(other)?)),
            },
            TypeKind::Optional(inner) => self.type_size(inner)?.checked_add(1),
            TypeKind::Tuple(parts) => parts.iter().try_fold(1u128, |total, part| {
                total.checked_mul(self.type_size(part)?)
            }),
            TypeKind::Record(fields) => fields.iter().try_fold(1u128, |total, (_, ty)| {
                total.checked_mul(self.type_size(ty)?)
            }),
            _ => None,
        }
    }
}

/// `("g", "u")` for the atom `g[u].len`.
pub(crate) fn row_of(atom: &str) -> Option<(&str, &str)> {
    atom.strip_suffix("].len")?.split_once('[')
}

/// `xs` for `xs`: a change there can change `xs.len`.
pub(crate) fn root(e: &Expr) -> Option<&str> {
    match &e.kind {
        ExprKind::Name(n) => Some(n),
        ExprKind::Paren(inner) => root(inner),
        _ => None,
    }
}

/// `g` for `g` or `g[u]`: a change there can change a `g[u].len`.
pub(crate) fn row_root(e: &Expr) -> Option<&str> {
    match &e.kind {
        ExprKind::Paren(inner) => row_root(inner),
        ExprKind::Index { base, indices } if indices.len() == 1 => root(base),
        _ => root(e),
    }
}

/// Whether every built-in list or string method called `method` leaves its
/// receiver as it was, so a call to it changes no length.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn keeps_len(method: &str) -> bool {
    use cpl_check::{builtin_members, Receiver};
    // The catalog only says whether a method mutates; these only move or
    // overwrite elements.
    const IN_PLACE: [&str; 4] = ["sort", "reverse", "swap", "fill"];
    let mut members = [Receiver::List, Receiver::Str]
        .into_iter()
        .flat_map(builtin_members)
        .filter(|m| m.name == method)
        .peekable();
    members.peek().is_some() && members.all(|m| !m.mutates || IN_PLACE.contains(&m.name))
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn keeps_len(_: &str) -> bool {
    false
}

/// `T` in `read<T>()`.
pub(crate) fn read_type(init: &Expr) -> Option<&Type> {
    let ExprKind::Call { callee, args, .. } = &init.kind else {
        return None;
    };
    let ExprKind::Generic { base, args: tys } = &callee.kind else {
        return None;
    };
    match (&base.kind, tys.as_slice()) {
        (ExprKind::Name(n), [ty]) if n == "read" && args.is_empty() => Some(ty),
        _ => None,
    }
}

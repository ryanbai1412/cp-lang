//! The walk over a program that records, in order, every field it reads.

use super::*;

pub(super) struct Builder<'a> {
    pub(super) src: &'a str,
    pub(super) base: u32,
    /// `domain V = lo..=hi` as inclusive bounds.
    pub(super) domains: HashMap<String, (Expr, Expr)>,
    /// Each alias with the base of the item that declares it, which its
    /// spans are relative to.
    pub(super) aliases: HashMap<String, (u32, Type)>,
    /// Each record type `read()` reads field by field: a struct's fields,
    /// or an `@tags` enum's variants each with the letter it is written as.
    pub(super) records: HashMap<String, (u32, Record)>,
    /// How the `read()` being built writes `none`, from `read(none: x)`.
    pub(super) none: Option<i128>,
    /// Each fn whose call reads the same values wherever it is, with the
    /// types it reads in order ([`values`]).
    pub(super) values: HashMap<String, (u32, Vec<Type>)>,
    /// The element type of each array declared once with its type, so
    /// `a[i] = read()` reads one of them ([`elems`]).
    pub(super) elems: HashMap<String, (u32, Type)>,
    pub(super) spec: InputSpec,
}

#[derive(Clone)]
pub(super) enum Record {
    Struct(Vec<Type>),
    /// A struct another module declares, its fields already shaped.
    Shaped(Vec<Shape>),
    /// Each variant's tag and its fields' types.
    Enum(Vec<(String, Vec<Type>)>),
}

impl Record {
    /// An enum reads one token, its variant's tag: the variant's `@tag(..)`,
    /// its place in `@tags(..)` above the enum, or else its name.
    pub(super) fn tagged(stmt: &Stmt, decl: &cpl_syntax::ast::EnumDecl) -> Option<Record> {
        let tags = stmt.decorators.iter().find(|d| d.name.text == "tags");
        if tags.is_some_and(|tags| tags.args.len() != decl.variants.len()) {
            return None;
        }
        let word = |expr: &Expr| match &expr.kind {
            ExprKind::Str(pieces) => match pieces.as_slice() {
                [cpl_syntax::ast::StrPiece::Text(text)] => Some(text.clone()),
                _ => None,
            },
            ExprKind::Char(c) => Some(c.to_string()),
            ExprKind::Int(n) => Some(n.to_string()),
            _ => None,
        };
        let variants = decl
            .variants
            .iter()
            .enumerate()
            .map(|(at, variant)| {
                let tag = match (&variant.tag, tags) {
                    (Some(tag), _) => word(tag)?,
                    (None, Some(tags)) => word(&tags.args[at].value)?,
                    (None, None) => variant.name.text.clone(),
                };
                let fields = variant.fields.iter().map(|(_, ty)| ty.clone()).collect();
                Some((tag, fields))
            })
            .collect::<Option<Vec<(String, Vec<Type>)>>>()?;
        let words = variants.iter().map(|(tag, _)| tag);
        let fine = |w: &String| !w.is_empty() && !w.contains(char::is_whitespace);
        (words.clone().all(fine)
            && words.collect::<std::collections::HashSet<_>>().len() == variants.len())
        .then_some(Record::Enum(variants))
    }
}

impl Builder<'_> {
    pub(super) fn error(&self, span: Span, message: String) -> SpecError {
        SpecError {
            span: span.shift(self.base),
            message,
        }
    }

    /// `lo..hi` or `lo..=hi` as inclusive bounds.
    pub(super) fn inclusive(lo: &Expr, hi: &Expr, inclusive: bool) -> (Expr, Expr) {
        let hi = if inclusive {
            hi.clone()
        } else {
            binary(BinOp::Sub, hi.clone(), int(1))
        };
        (lo.clone(), hi)
    }

    /// The bounds of a type used as an array index: a domain or a range.
    pub(super) fn index(&self, ty: &Type) -> Result<(Expr, Expr), SpecError> {
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
    pub(super) fn cell<'t>(&self, ty: &'t Type) -> Option<&'t Type> {
        match &ty.kind {
            TypeKind::Named { name, args } if name.text == "List" && args.len() == 1 => {
                Some(&args[0])
            }
            TypeKind::Array { index, elem } if index.len() == 1 => Some(elem),
            _ => None,
        }
    }

    pub(super) fn shape(&mut self, ty: &Type) -> Result<Shape, SpecError> {
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
                    // A whole number is an `F64` too; drawing fractions
                    // waits for a fixed-point value (decision 25.21).
                    "Int" | "I128" | "F64" => {
                        return Ok(Shape::Int {
                            lo: int(DEFAULT_INT.0),
                            hi: int(DEFAULT_INT.1),
                        })
                    }
                    "U64" => {
                        return Ok(Shape::Int {
                            lo: int(0),
                            hi: int(DEFAULT_INT.1),
                        })
                    }
                    "Char" => return Ok(Shape::Char { lo: b'a', hi: b'z' }),
                    "Bool" => return Ok(Shape::Bool),
                    "Str" => return Ok(Shape::WORD),
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
                    let module =
                        cpl_check::std_module(&name.text).ok_or_else(|| unsupported(self))?;
                    let module = ["std".to_string(), module.to_string()];
                    return match std_structs(&module, &[name.text.as_str()], false).pop() {
                        Some((_, parts)) => self.record(&name.text, &Record::Shaped(parts)),
                        None => Err(unsupported(self)),
                    };
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
            // `Range<T>` reads its two ends `l r` of `T`, `l <= r`.
            TypeKind::Named { name, args } if name.text == "Range" && args.len() == 1 => {
                let Shape::Int { lo, hi } = self.shape(&args[0])? else {
                    return Err(unsupported(self));
                };
                let start = String::from("(range start)");
                let at_start = Expr {
                    kind: ExprKind::Name(start.clone()),
                    span: args[0].span,
                };
                Ok(Shape::Tuple {
                    parts: vec![
                        Shape::Int { lo, hi: hi.clone() },
                        Shape::Int { lo: at_start, hi },
                    ],
                    binds: vec![vec![Some(start)], Vec::new()],
                })
            }
            // `Mod<P>` reads a residue, `0..P`.
            TypeKind::Named { name, args } if name.text == "Mod" && args.len() == 1 => {
                let p = match &args[0].kind {
                    TypeKind::Value(p) => (**p).clone(),
                    TypeKind::Named { name, args } if args.is_empty() => Expr {
                        kind: ExprKind::Name(name.text.clone()),
                        span: name.span,
                    },
                    _ => return Err(unsupported(self)),
                };
                let hi = binary(BinOp::Sub, p, int(1));
                names_in(&hi, &mut self.spec.sizes);
                Ok(Shape::Int { lo: int(0), hi })
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
                Ok(Shape::WORD)
            }
            // `Str of "01"`, `Str(len: 1..=n)`: a word over that alphabet,
            // of a length in that range.
            TypeKind::ConstrainedStr { alphabet, len } => {
                let alphabet = match alphabet {
                    Some(text) => {
                        let mut bytes: Vec<u8> = text.bytes().collect();
                        bytes.sort_unstable();
                        bytes.dedup();
                        if bytes.is_empty() || bytes.iter().any(|b| b.is_ascii_whitespace()) {
                            return Err(unsupported(self));
                        }
                        Some(bytes.into())
                    }
                    None => None,
                };
                let len = match len {
                    Some(len) => {
                        let (lo, hi) = self.index(len)?;
                        names_in(&lo, &mut self.spec.sizes);
                        names_in(&hi, &mut self.spec.sizes);
                        Some((lo, hi))
                    }
                    None => None,
                };
                Ok(Shape::Str { alphabet, len })
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
                        index: None,
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
    pub(super) fn perm(&mut self, ty: &Type) -> Option<Shape> {
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
    pub(super) fn tree(&mut self, ty: &Type) -> Option<Shape> {
        let (_, lo, hi, extra) = self.edges(ty)?;
        Some(Shape::Tree { lo, hi, extra })
    }

    /// `[E](u: V, v: V, ..)` as `kind` of graph on `V` with an edge for
    /// each value of `E`.
    pub(super) fn graph(&mut self, ty: &Type, kind: GraphKind) -> Option<Shape> {
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
    pub(super) fn edges(&mut self, ty: &Type) -> Option<(Expr, Expr, Expr, Vec<Shape>)> {
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
    pub(super) fn pass(
        &mut self,
        stmt: &Stmt,
        outer: &HashSet<String>,
    ) -> Result<Option<Shape>, SpecError> {
        let StmtKind::For {
            pat, iter, body, ..
        } = &stmt.kind
        else {
            return Ok(None);
        };
        let Some((mut lo, mut hi)) = self.over(iter) else {
            return Ok(None);
        };
        let mut stmts = body.stmts.as_slice();
        // `if v == V.first: continue` first in the body: the passes that
        // read start at the second.
        if let (PatternKind::Name(name), Some(first)) = (&pat.kind, stmts.first()) {
            let skipped = Self::skipped_end(first, name, iter, &lo, &hi);
            match skipped {
                Some(true) => lo = binary(BinOp::Add, lo, int(1)),
                Some(false) => hi = binary(BinOp::Sub, hi, int(1)),
                None => {}
            }
            if skipped.is_some() {
                stmts = &stmts[1..];
            }
        }
        let mut used = HashSet::new();
        names_in(&lo, &mut used);
        names_in(&hi, &mut used);
        if !used.is_disjoint(outer) {
            return Ok(None);
        }
        let mut inside = outer.clone();
        // The loop's own name is each pass's position, so the reads
        // may use it; any other pattern is left unknown.
        let index = match &pat.kind {
            PatternKind::Name(name) => {
                inside.remove(name);
                Some(name.clone())
            }
            _ => {
                bind_names(pat, &mut inside);
                None
            }
        };
        let mut parts = Vec::new();
        let mut binds = Vec::new();
        if !self.rows(stmts, &mut inside, &mut parts, &mut binds)? {
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
            len: binary(BinOp::Add, binary(BinOp::Sub, hi, lo.clone()), int(1)),
            elem: Box::new(elem),
            index: index.map(|name| (name, lo)),
        }))
    }

    /// Whether `stmt` is `if name == first: continue` (`Some(true)`) or
    /// the same at the last pass (`Some(false)`), for a loop over `iter`
    /// from `lo` to `hi`.
    pub(super) fn skipped_end(
        stmt: &Stmt,
        name: &str,
        iter: &Expr,
        lo: &Expr,
        hi: &Expr,
    ) -> Option<bool> {
        let StmtKind::Expr(Expr {
            kind:
                ExprKind::If {
                    branches,
                    otherwise: None,
                },
            ..
        }) = &stmt.kind
        else {
            return None;
        };
        let [(Cond::Expr(cond), then)] = branches.as_slice() else {
            return None;
        };
        if !matches!(
            then.stmts.as_slice(),
            [Stmt {
                kind: StmtKind::Continue(None),
                ..
            }]
        ) {
            return None;
        }
        Self::end_compared(cond, name, iter, lo, hi)
    }

    /// Whether `cond` is `name == first` (`Some(true)`) or `name == last`
    /// (`Some(false)`) for `name` running over `iter` from `lo` to `hi`.
    pub(super) fn end_compared(
        cond: &Expr,
        name: &str,
        iter: &Expr,
        lo: &Expr,
        hi: &Expr,
    ) -> Option<bool> {
        let ExprKind::Compare { first, rest } = &cond.kind else {
            return None;
        };
        let [(CmpOp::Eq, end)] = rest.as_slice() else {
            return None;
        };
        let is_name = |expr: &Expr| matches!(&expr.kind, ExprKind::Name(at) if at == name);
        let end = match (is_name(first), is_name(end)) {
            (true, false) => end,
            (false, true) => &**first,
            _ => return None,
        };
        let ends = |field: &str| match (&end.kind, &iter.kind) {
            (ExprKind::Field { base, name }, ExprKind::Name(domain)) => {
                name.text == field && matches!(&base.kind, ExprKind::Name(d) if d == domain)
            }
            _ => false,
        };
        if same(end, lo) || ends("first") {
            Some(true)
        } else if same(end, hi) || ends("last") {
            Some(false)
        } else {
            None
        }
    }

    /// The bounds of a table's one domain: a range or a declared domain.
    pub(super) fn cells(&self, domain: &Type) -> Option<(Expr, Expr)> {
        match &domain.kind {
            TypeKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
            } => Some(Builder::inclusive(lo, hi, *inclusive)),
            TypeKind::Named { name, args } if args.is_empty() => {
                self.domains.get(&name.text).cloned()
            }
            _ => None,
        }
    }

    /// For `[D] i => if i == D.first: x else: y` where `x` does not read,
    /// `y` and whether the first cell (`true`) or the last is left out.
    pub(super) fn skipped_cell<'e>(
        &self,
        domains: &[Type],
        params: &[Pattern],
        body: &'e Expr,
    ) -> Option<(&'e Expr, bool)> {
        let (
            [domain],
            [Pattern {
                kind: PatternKind::Name(name),
                ..
            }],
        ) = (domains, params)
        else {
            return None;
        };
        let ExprKind::If {
            branches,
            otherwise: Some(otherwise),
        } = &body.kind
        else {
            return None;
        };
        let [(Cond::Expr(cond), then)] = branches.as_slice() else {
            return None;
        };
        let only = |block: &'e Block| match block.stmts.as_slice() {
            [Stmt {
                kind: StmtKind::Expr(expr),
                ..
            }] => Some(expr),
            _ => None,
        };
        let (then, otherwise) = (only(then)?, only(otherwise)?);
        if self.reads_in(then) {
            return None;
        }
        let (lo, hi) = self.cells(domain)?;
        let iter = match &domain.kind {
            TypeKind::Named { name, .. } => Expr {
                kind: ExprKind::Name(name.text.clone()),
                span: domain.span,
            },
            _ => int(0),
        };
        let first = Self::end_compared(cond, name, &iter, &lo, &hi)?;
        Some((otherwise, first))
    }

    /// The type a call whose arguments are all bare `read()`s gives
    /// them: `Graph<V, E>.undirected(read())` reads `[E](V, V)`, and a
    /// struct `Event(read(), read())` reads an `Event`.
    pub(super) fn read_as(&self, expr: &Expr) -> Option<Type> {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return None;
        };
        let bare = |arg: &cpl_syntax::ast::Arg| {
            arg.name.is_none()
                && matches!(&arg.value.kind, ExprKind::Call { callee, args }
                    if args.is_empty() && matches!(&callee.kind, ExprKind::Name(name) if name == "read"))
        };
        if args.is_empty() || !args.iter().all(bare) {
            return None;
        }
        let named = |name: &cpl_syntax::ast::Name, args: Vec<Type>| Type {
            kind: TypeKind::Named {
                name: name.clone(),
                args,
            },
            span: expr.span,
        };
        match &callee.kind {
            ExprKind::Field { base, name } if args.len() == 1 => {
                let ExprKind::Generic { base, args: tys } = &base.kind else {
                    return None;
                };
                let (ExprKind::Name(graph), [v, e]) = (&base.kind, tys.as_slice()) else {
                    return None;
                };
                (graph == "Graph" && matches!(name.text.as_str(), "undirected" | "directed")).then(
                    || Type {
                        kind: TypeKind::Array {
                            index: vec![e.clone()],
                            elem: Box::new(Type {
                                kind: TypeKind::Tuple(vec![v.clone(), v.clone()]),
                                span: expr.span,
                            }),
                        },
                        span: expr.span,
                    },
                )
            }
            ExprKind::Name(record) => match self.records.get(record) {
                Some((_, Record::Struct(fields))) if fields.len() == args.len() => Some(named(
                    &cpl_syntax::ast::Name {
                        text: record.clone(),
                        span: callee.span,
                    },
                    Vec::new(),
                )),
                _ => None,
            },
            _ => None,
        }
    }

    /// The typed reads `expr` makes on every evaluation, in order, as
    /// `read<[n - 1](V, V)>()` inside `Graph.undirected(..)` or
    /// `[0..n] _ => read::<Int>() - 1`. False when a read is untyped or
    /// sits where it may run another number of times (a branch, a
    /// closure, a filtered or non-range comprehension).
    pub(super) fn inline(&mut self, expr: &Expr, out: &mut Vec<Shape>) -> Result<bool, SpecError> {
        if let Some((ty, None)) = typed_read(None, expr) {
            out.push(self.shape(&ty)?);
            return Ok(true);
        }
        if matches!(&expr.kind, ExprKind::Call { callee, .. } if matches!(&callee.kind, ExprKind::Name(name) if name == "read"))
        {
            return Ok(false);
        }
        if let Some(ty) = self.read_as(expr) {
            out.push(self.shape(&ty)?);
            return Ok(true);
        }
        if let ExprKind::Call { callee, args } = &expr.kind {
            if let (ExprKind::Name(name), []) = (&callee.kind, args.as_slice()) {
                if let Some((base, types)) = self.values.get(name).cloned() {
                    let outer = std::mem::replace(&mut self.base, base);
                    let shapes: Result<Vec<_>, _> = types.iter().map(|ty| self.shape(ty)).collect();
                    self.base = outer;
                    out.extend(shapes?);
                    return Ok(true);
                }
            }
        }
        let mut all = |this: &mut Self, exprs: &mut dyn Iterator<Item = &Expr>| {
            for expr in exprs {
                if !this.inline(expr, out)? {
                    return Ok(false);
                }
            }
            Ok(true)
        };
        let each = |this: &mut Self, body: &Expr, len: Expr, out: &mut Vec<Shape>| {
            let mut inner = Vec::new();
            if !this.inline(body, &mut inner)? {
                return Ok(false);
            }
            if inner.is_empty() {
                return Ok(true);
            }
            names_in(&len, &mut this.spec.sizes);
            let elem = match inner.len() {
                1 => inner.pop().expect("one shape"),
                _ => Shape::tuple(inner),
            };
            out.push(Shape::Array {
                len,
                elem: Box::new(elem),
                index: None,
            });
            Ok(true)
        };
        match &expr.kind {
            ExprKind::Unary { expr, .. }
            | ExprKind::Field { base: expr, .. }
            | ExprKind::Generic { base: expr, .. }
            | ExprKind::Paren(expr)
            | ExprKind::Try(expr)
            | ExprKind::Unwrap(expr)
            | ExprKind::Mut(expr) => self.inline(expr, out),
            ExprKind::Binary { op, lhs, rhs, .. } if !matches!(op, BinOp::And | BinOp::Or) => {
                all(self, &mut [&**lhs, &**rhs].into_iter())
            }
            // A chain `a < b < c` may stop before its later operands.
            ExprKind::Compare { first, rest } if rest.len() == 1 => {
                all(self, &mut [&**first, &rest[0].1].into_iter())
            }
            ExprKind::Call { callee, args } => all(
                self,
                &mut std::iter::once(&**callee).chain(args.iter().map(|arg| &arg.value)),
            ),
            ExprKind::Index { base, indices } => {
                all(self, &mut std::iter::once(&**base).chain(indices))
            }
            ExprKind::Tuple(exprs) | ExprKind::List(exprs) => all(self, &mut exprs.iter()),
            ExprKind::Record(fields) => all(self, &mut fields.iter().map(|(_, expr)| expr)),
            ExprKind::Table {
                domains,
                params,
                body,
            } if self.skipped_cell(domains, params, body).is_some() => {
                let (body, _) = self
                    .skipped_cell(domains, params, body)
                    .expect("tested above");
                let (lo, hi) = self.cells(&domains[0]).expect("tested above");
                each(self, body, binary(BinOp::Sub, hi, lo), out)
            }
            ExprKind::Table {
                domains,
                params,
                body,
            } if domains.len() == 1
                && params
                    .iter()
                    .all(|pat| matches!(pat.kind, PatternKind::Wildcard)) =>
            {
                let Shape::Array { len, .. } = self.shape(&Type {
                    kind: TypeKind::Array {
                        index: domains.clone(),
                        elem: Box::new(Type {
                            kind: TypeKind::Named {
                                name: cpl_syntax::ast::Name {
                                    text: "Bool".into(),
                                    span: expr.span,
                                },
                                args: Vec::new(),
                            },
                            span: expr.span,
                        }),
                    },
                    span: expr.span,
                })?
                else {
                    return Ok(false);
                };
                each(self, body, len, out)
            }
            ExprKind::Match { scrutinee, .. } => {
                Ok(self.inline(scrutinee, out)? && !self.reads_after(scrutinee, expr.span))
            }
            ExprKind::Comprehension { body, clauses } => match clauses.as_slice() {
                [clause]
                    if clause.conds.is_empty()
                        && matches!(clause.pat.kind, PatternKind::Wildcard) =>
                {
                    let ExprKind::Range {
                        lo: Some(lo),
                        hi: Some(hi),
                        inclusive,
                        step: None,
                    } = &clause.iter.kind
                    else {
                        return Ok(!self.reads_in(expr));
                    };
                    if self.reads_in(&clause.iter) {
                        return Ok(false);
                    }
                    let (lo, hi) = Builder::inclusive(lo, hi, *inclusive);
                    each(
                        self,
                        body,
                        binary(BinOp::Add, binary(BinOp::Sub, hi, lo), int(1)),
                        out,
                    )
                }
                _ => Ok(!self.reads_in(expr)),
            },
            _ => Ok(!self.reads_in(expr)),
        }
    }

    pub(super) fn reads_in(&self, expr: &Expr) -> bool {
        self.reads_code(&self.src[span_range(expr.span.shift(self.base))])
    }

    /// True when `code` reads: it calls `read`, or a fn of [`values`].
    pub(super) fn reads_code(&self, code: &str) -> bool {
        reads_inside(code)
            || (!self.values.is_empty()
                && cpl_syntax::lex(code).tokens.windows(2).any(|pair| {
                    pair[0].kind == cpl_syntax::TokenKind::Ident
                        && pair[1].kind == cpl_syntax::TokenKind::LParen
                        && self.values.contains_key(
                            &code[pair[0].span.start as usize..pair[0].span.end as usize],
                        )
                }))
    }

    /// True when `whole` reads after its part `first` ends.
    pub(super) fn reads_after(&self, first: &Expr, whole: Span) -> bool {
        let (first, whole) = (first.span.shift(self.base), whole.shift(self.base));
        self.reads_code(&self.src[first.end as usize..whole.end as usize])
    }

    /// The expression of a statement that only evaluates one: `let`,
    /// an assignment, or a bare expression.
    pub(super) fn evaluated<'s>(&self, stmt: &'s Stmt) -> Option<&'s Expr> {
        match &stmt.kind {
            StmtKind::Let {
                init: Some(init), ..
            } => Some(init),
            // `a[read()] = read()` has no order the reads are sure to run in.
            StmtKind::Assign { target, value, .. } => match self.reads_in(target) {
                false => Some(value),
                true => (!self.reads_in(value)).then_some(target),
            },
            StmtKind::Expr(expr) => Some(expr),
            StmtKind::For { iter, .. } => (!self.reads_after(iter, stmt.span)).then_some(iter),
            _ => None,
        }
    }

    /// The fields `stmt` reads through [`Builder::inline`], or `None`.
    pub(super) fn inlined(&mut self, stmt: &Stmt) -> Result<Option<Vec<Shape>>, SpecError> {
        let Some(expr) = self.evaluated(stmt) else {
            return Ok(None);
        };
        let mut shapes = Vec::new();
        Ok(self.inline(expr, &mut shapes)?.then_some(shapes))
    }

    pub(super) fn reads(&self, stmt: &Stmt) -> bool {
        self.reads_code(&self.src[span_range(stmt.span.shift(self.base))])
    }

    /// The element type `a[i] = read()` reads: `a`'s, when its array
    /// type is declared.
    pub(super) fn elem_read(&self, target: &Expr, value: &Expr) -> Option<(u32, Type)> {
        let ExprKind::Call { callee, args } = &value.kind else {
            return None;
        };
        let (ExprKind::Name(read), []) = (&callee.kind, args.as_slice()) else {
            return None;
        };
        let ExprKind::Index { base, .. } = &target.kind else {
            return None;
        };
        let ExprKind::Name(name) = &base.kind else {
            return None;
        };
        (read == "read").then(|| self.elems.get(name).cloned())?
    }

    /// The reads of `stmts`, one pass of a loop or one arm of a choice,
    /// added to `parts`. False when a field cannot describe them.
    pub(super) fn rows(
        &mut self,
        stmts: &[Stmt],
        inside: &mut HashSet<String>,
        parts: &mut Vec<Shape>,
        binds: &mut Vec<Vec<Option<String>>>,
    ) -> Result<bool, SpecError> {
        let last = stmts.iter().rposition(|stmt| self.reads(stmt));
        for (at, stmt) in stmts.iter().enumerate() {
            let jumps = Jumps::of(stmt);
            if jumps.leaves() || (jumps.skips() && last.is_some_and(|last| at < last)) {
                return Ok(false);
            }
            match &stmt.kind {
                StmtKind::Let {
                    pat,
                    ty,
                    init: Some(init),
                    ..
                } if typed_read(ty.as_ref(), init).is_some() => {
                    let (ty, none) = typed_read(ty.as_ref(), init).expect("tested above");
                    let mut used = HashSet::new();
                    Sizes(&mut used).of_type(&ty);
                    type_names(&ty, &mut used);
                    if !used.is_disjoint(inside) {
                        return Ok(false);
                    }
                    let shape = self.shape_written(&ty, none)?;
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
                    // A domain over values the pass computes is unknown
                    // here, like those values: only a read that uses it
                    // is refused.
                    if !used.is_disjoint(inside) && read.is_none() {
                        self.domains.remove(&name.text);
                        inside.insert(name.text.clone());
                        continue;
                    }
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
                    None => match self.inlined(stmt)? {
                        Some(shapes) => {
                            binds.extend(shapes.iter().map(|_| Vec::new()));
                            parts.extend(shapes);
                        }
                        None => return Ok(false),
                    },
                },
                StmtKind::Expr(expr)
                    if self.reads(stmt)
                        && matches!(expr.kind, ExprKind::If { .. } | ExprKind::Match { .. }) =>
                {
                    if !self.choice(expr, inside, parts, binds)? {
                        let Some(shapes) = self.inlined(stmt)? else {
                            return Ok(false);
                        };
                        binds.extend(shapes.iter().map(|_| Vec::new()));
                        parts.extend(shapes);
                    }
                }
                StmtKind::Assign {
                    target,
                    op: AssignOp::Set,
                    value,
                } if self.elem_read(target, value).is_some() => {
                    let (base, ty) = self.elem_read(target, value).expect("tested above");
                    let mut used = HashSet::new();
                    Sizes(&mut used).of_type(&ty);
                    type_names(&ty, &mut used);
                    if !used.is_disjoint(inside) {
                        return Ok(false);
                    }
                    let outer = std::mem::replace(&mut self.base, base);
                    let shape = self.shape(&ty);
                    self.base = outer;
                    binds.push(Vec::new());
                    parts.push(shape?);
                }
                _ if self.reads(stmt) => match self.inlined(stmt)? {
                    Some(shapes) => {
                        binds.extend(shapes.iter().map(|_| Vec::new()));
                        parts.extend(shapes);
                        if let StmtKind::Let { pat, .. } = &stmt.kind {
                            bind_names(pat, inside);
                        }
                    }
                    None => return Ok(false),
                },
                StmtKind::Let { pat, .. } => bind_names(pat, inside),
                _ => {}
            }
        }
        Ok(true)
    }

    /// An `if` or `match` on a tag read above it, each branch reading
    /// its own values: `if op == 1: let (l, r) = read() else: let i = read()`.
    /// The tag is a number read alone or first in a tuple; what is read
    /// between it and the branch is read by every branch, before its own.
    pub(super) fn choice(
        &mut self,
        expr: &Expr,
        inside: &mut HashSet<String>,
        parts: &mut Vec<Shape>,
        binds: &mut Vec<Vec<Option<String>>>,
    ) -> Result<bool, SpecError> {
        let scalar = |shape: &Shape| {
            matches!(
                shape,
                Shape::Int { .. } | Shape::Char { .. } | Shape::Bool | Shape::Str { .. }
            )
        };
        for at in (0..parts.len()).rev() {
            let Some(Some(name)) = binds[at].first().cloned() else {
                continue;
            };
            let (tag, rest) = match &parts[at] {
                shape if scalar(shape) && binds[at].len() == 1 => (shape.clone(), None),
                Shape::Tuple {
                    parts: inner,
                    binds: inner_binds,
                } if inner.len() > 1 && inner.len() == binds[at].len() && scalar(&inner[0]) => {
                    let rest = Shape::Tuple {
                        parts: inner[1..].to_vec(),
                        binds: inner_binds.get(1..).unwrap_or_default().to_vec(),
                    };
                    (inner[0].clone(), Some((rest, binds[at][1..].to_vec())))
                }
                _ => continue,
            };
            let rebound = binds[at][1..]
                .iter()
                .chain(binds[at + 1..].iter().flatten())
                .any(|n| n.as_deref() == Some(name.as_str()));
            if rebound {
                return Ok(false);
            }
            let Some((tag, mut arms)) = self.arms_on(expr, inside, &name, tag)? else {
                continue;
            };
            let mut between: Vec<(Shape, Vec<Option<String>>)> = rest.into_iter().collect();
            between.extend(parts.drain(at + 1..).zip(binds.drain(at + 1..)));
            parts.pop();
            binds.pop();
            if !between.is_empty() && arms.iter().all(|arm| arm.tags.is_some()) {
                arms.push(Arm {
                    tags: None,
                    parts: Vec::new(),
                    binds: Vec::new(),
                });
            }
            for arm in &mut arms {
                let (shapes, names): (Vec<_>, Vec<_>) = between.iter().cloned().unzip();
                arm.parts.splice(0..0, shapes);
                arm.binds.splice(0..0, names);
            }
            inside.insert(name.clone());
            parts.push(Shape::Choice {
                tag: Box::new(tag),
                name,
                arms,
            });
            binds.push(Vec::new());
            return Ok(true);
        }
        Ok(false)
    }

    /// The top-level fields from a tag on, made one choice by the branch
    /// `expr` on that tag; false when `expr` is not one.
    pub(super) fn top_choice(&mut self, expr: &Expr) -> Result<bool, SpecError> {
        if !matches!(expr.kind, ExprKind::If { .. } | ExprKind::Match { .. }) {
            return Ok(false);
        }
        // A tag named twice is ambiguous: which one the branch tests
        // depends on scope the fields no longer carry.
        let mut named = HashSet::new();
        let names = self
            .spec
            .fields
            .iter()
            .flat_map(|field| field.names.iter().flatten());
        if !names.into_iter().all(|name| named.insert(name)) {
            return Ok(false);
        }
        let fields = std::mem::take(&mut self.spec.fields);
        let (mut parts, mut binds): (Vec<_>, Vec<_>) = fields
            .iter()
            .map(|field| (field.shape.clone(), field.names.clone()))
            .unzip();
        let mut inside = HashSet::new();
        let chose = self.choice(expr, &mut inside, &mut parts, &mut binds);
        match chose {
            Ok(true) => {
                let kept = parts.len() - 1;
                let line = fields[kept].line;
                self.spec.fields = fields;
                self.spec.fields.truncate(kept);
                self.spec.fields.push(Field {
                    names: Vec::new(),
                    shape: parts.pop().expect("the choice"),
                    line,
                });
                Ok(true)
            }
            other => {
                self.spec.fields = fields;
                other
            }
        }
    }

    /// The arms of `expr` as a choice on the tag `name`, and the tag's
    /// shape; `None` when `expr` is not a choice on it.
    pub(super) fn arms_on(
        &mut self,
        expr: &Expr,
        inside: &HashSet<String>,
        name: &str,
        tag: Shape,
    ) -> Result<Option<(Shape, Vec<Arm>)>, SpecError> {
        let consts = self.spec.consts.clone();
        // A `Str` tag is compared with words: each is its position in
        // the order the arms name them.
        let words = std::cell::RefCell::new(Vec::<String>::new());
        let word = matches!(tag, Shape::Str { .. });
        let literal = |expr: &Expr| match &expr.kind {
            ExprKind::Str(pieces) if word => {
                let [StrPiece::Text(text)] = pieces.as_slice() else {
                    return None;
                };
                let mut words = words.borrow_mut();
                let at = words.iter().position(|w| w == text).unwrap_or_else(|| {
                    words.push(text.clone());
                    words.len() - 1
                });
                Some(at as i128)
            }
            _ if word => None,
            _ => eval(expr, &consts),
        };
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
                        return Ok(None);
                    };
                    match tags(cond, name, &literal) {
                        Some((values, false)) => ways.push((Some(values), Some(block))),
                        // `if x != 1: A else: B` is `B` for 1, else `A`.
                        Some((values, true)) if at == 0 && branches.len() == 1 => {
                            ways.push((Some(values), otherwise.as_ref()));
                            ways.push((None, Some(block)));
                        }
                        _ => return Ok(None),
                    }
                }
                if !matches!(ways.last(), Some((None, _))) {
                    ways.push((None, otherwise.as_ref()));
                }
            }
            ExprKind::Match { scrutinee, arms } if matches!(&scrutinee.kind, ExprKind::Name(n) if n == name) =>
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
                        return Ok(None);
                    }
                    let Some(tags) = values(&arm.pat, &literal) else {
                        return Ok(None);
                    };
                    ways.push((tags, Some(&arm.body)));
                }
            }
            _ => return Ok(None),
        }
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
                    return Ok(None);
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
            Shape::Str { .. } => {
                let mut words = words.into_inner();
                if words.is_empty()
                    || words
                        .iter()
                        .any(|w| w.is_empty() || w.contains(char::is_whitespace))
                {
                    return Ok(None);
                }
                // One more word for an arm that takes every other word.
                if arms.iter().any(|arm| arm.tags.is_none()) {
                    let mut other = String::from("x");
                    while words.contains(&other) {
                        other.push('x');
                    }
                    words.push(other);
                }
                Shape::Words(words.into())
            }
            tag => tag,
        };
        Ok(Some((tag, arms)))
    }

    /// The shape of `ty` read with `none` written as `none`.
    pub(super) fn shape_written(
        &mut self,
        ty: &Type,
        none: Option<i128>,
    ) -> Result<Shape, SpecError> {
        let outer = std::mem::replace(&mut self.none, none);
        let shape = self.shape(ty);
        self.none = outer;
        shape
    }

    /// A struct is its fields in order; an `@tags` enum is a choice on
    /// the letter its variant is written as, then that variant's fields.
    pub(super) fn record(&mut self, name: &str, record: &Record) -> Result<Shape, SpecError> {
        let parts = |this: &mut Self, types: &[Type]| -> Result<Vec<Shape>, SpecError> {
            types.iter().map(|ty| this.shape(ty)).collect()
        };
        match record {
            Record::Struct(fields) => {
                let parts = parts(self, fields)?;
                let binds = vec![Vec::new(); parts.len()];
                Ok(Shape::Tuple { parts, binds })
            }
            Record::Shaped(parts) => Ok(Shape::Tuple {
                binds: vec![Vec::new(); parts.len()],
                parts: parts.clone(),
            }),
            Record::Enum(variants) => {
                if variants.is_empty() {
                    return Err(
                        self.error(Span::new(0, 0), format!("`{name}` has no variants to read"))
                    );
                }
                // One-letter tags are a `Char`; longer ones are words, each
                // its position in the enum.
                let letters: Option<Vec<u8>> = variants
                    .iter()
                    .map(|(tag, _)| match tag.as_bytes() {
                        [b] if b.is_ascii_graphic() => Some(*b),
                        _ => None,
                    })
                    .collect();
                let mut arms = Vec::new();
                for (at, (_, fields)) in variants.iter().enumerate() {
                    let parts = parts(self, fields)?;
                    let binds = vec![Vec::new(); parts.len()];
                    let tag = letters.as_ref().map_or(at as i128, |l| i128::from(l[at]));
                    arms.push(Arm {
                        tags: Some(vec![tag]),
                        parts,
                        binds,
                    });
                }
                let tag = match letters {
                    Some(letters) => Shape::Char {
                        lo: *letters.iter().min().expect("variants"),
                        hi: *letters.iter().max().expect("variants"),
                    },
                    None => Shape::Words(variants.iter().map(|(tag, _)| tag.clone()).collect()),
                };
                Ok(Shape::Choice {
                    tag: Box::new(tag),
                    name: format!("{name}.tag"),
                    arms,
                })
            }
        }
    }

    /// The first and last value a loop goes over: `lo..hi`, `lo..=hi`
    /// or a domain declared above.
    pub(super) fn over(&self, iter: &Expr) -> Option<(Expr, Expr)> {
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
    pub(super) fn names(pat: &Pattern, shape: &Shape) -> Vec<Option<String>> {
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

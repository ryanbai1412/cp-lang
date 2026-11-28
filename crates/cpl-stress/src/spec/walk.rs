//! The walk over a program that records, in order, every field it reads.

use super::*;

pub(super) struct Builder<'a> {
    pub(super) src: &'a str,
    pub(super) base: u32,
    /// Inside a std type's declaration: its module, whose names its
    /// fields' types are, and what its generics stand for.
    pub(super) std_scope: Option<&'static str>,
    pub(super) bound: Rc<HashMap<String, Bound<'a>>>,
    /// `domain V = lo..=hi` as inclusive bounds.
    pub(super) domains: HashMap<String, (Expr, Expr)>,
    /// `domain Cell = (Row, Col)`: the domains a product domain pairs.
    pub(super) products: HashMap<String, Vec<Type>>,
    /// Domains declared by a count alone (`domain E = [m]`), which
    /// `io.base` shifts. (R10.2.2)
    pub(super) counted: HashSet<String>,
    /// The `io.base` a read here writes counted domains in.
    pub(super) io_base: i128,
    /// Each enum variant's enum and position in it.
    pub(super) variants: HashMap<String, (String, usize)>,
    /// Each alias with the base of the item that declares it, which its
    /// spans are relative to.
    pub(super) aliases: HashMap<String, (u32, Type)>,
    /// What `@stress(..)` says each alias above makes.
    pub(super) alias_made: HashMap<String, Made>,
    /// Each record type `read()` reads field by field: a struct's fields,
    /// or an `@tags` enum's variants each with the letter it is written as.
    pub(super) records: HashMap<String, (u32, Record)>,
    /// How the `read()` being built writes `none`, from `read(none: x)`.
    pub(super) none: Option<i128>,
    /// A read above took the input to its end, so a `?T` read finds
    /// `None` (R10.2.10).
    pub(super) ended: bool,
    /// Each fn whose call reads the same values wherever it is, with the
    /// types it reads in order ([`values`]).
    pub(super) values: HashMap<String, (u32, Vec<Type>)>,
    /// The element type of each array declared once with its type, so
    /// `a[i] = read()` reads one of them ([`elems`]).
    pub(super) elems: HashMap<String, (u32, Type)>,
    /// What each bare `read()` or `ask(..)` reads, from checker facts,
    /// as the source of a `let _: T = read()`, and which expressions are
    /// characters ([`read_types`]).
    pub(super) read_types: &'a ReadFacts,
    /// Domains whose positions are characters: `domain Letter = 'a'..='z'`.
    pub(super) char_domains: HashSet<String>,
    /// Each `fn f(..) -> T = body` whose body reads ([`bodies`]).
    pub(super) bodies: HashMap<&'a str, (u32, &'a Expr)>,
    /// Each block fn whose body reads ([`blocks`]).
    pub(super) blocks: HashMap<&'a str, (u32, &'a [Stmt], Vec<String>)>,
    /// Each `fn f() -> yields T` that reads until a sentinel ([`streams`]).
    pub(super) streams: HashMap<&'a str, (u32, &'a Stmt)>,
    /// The fns of [`Builder::bodies`] being inlined, outermost first.
    pub(super) inlining: Vec<&'a str>,
    pub(super) spec: InputSpec,
}

/// The type a std generic stands for, with the text and scope it is
/// written in.
#[derive(Clone)]
pub(super) struct Bound<'a> {
    src: &'a str,
    base: u32,
    ty: Type,
    scope: Option<&'static str>,
    outer: Rc<HashMap<String, Bound<'a>>>,
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
    /// Records whether `domain name = def` is over characters: the
    /// checker types its low bound `Char`.
    pub(super) fn note_char_domain(&mut self, name: &str, def: &Expr) {
        let over_chars = matches!(&def.kind, ExprKind::Range { lo: Some(lo), .. }
            if self.read_types.chars.contains(&lo.span.shift(self.base)));
        if over_chars {
            self.char_domains.insert(name.to_string());
        } else {
            self.char_domains.remove(name);
        }
    }

    /// The characters a word's element is: `Char`, or a domain over
    /// characters with constant bounds. A lone value of such a domain reads
    /// as its code point, so only a word reads its characters.
    fn letters(&mut self, elem: &Type) -> Option<(u8, u8)> {
        if let TypeKind::Named { name, args } = &elem.kind {
            if args.is_empty() && self.char_domains.contains(&name.text) {
                let (lo, hi) = self.domains.get(&name.text)?;
                let char = |end: &Expr| {
                    eval(end, &Env::new())
                        .and_then(|c| u8::try_from(c).ok())
                        .filter(u8::is_ascii_graphic)
                };
                return Some((char(lo)?, char(hi)?)).filter(|(lo, hi)| lo <= hi);
            }
        }
        match self.shape(elem) {
            Ok(Shape::Char { lo, hi }) => Some((lo, hi)),
            _ => None,
        }
    }

    pub(super) fn inclusive(lo: &Expr, hi: &Expr, inclusive: bool) -> (Expr, Expr) {
        let hi = if inclusive {
            hi.clone()
        } else {
            binary(BinOp::Sub, hi.clone(), int(1))
        };
        (lo.clone(), hi)
    }

    /// The bounds of a type used as an array index: a domain or a range.
    pub(super) fn index(&mut self, ty: &Type) -> Result<(Expr, Expr), SpecError> {
        match &ty.kind {
            TypeKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
            } => Ok(Self::inclusive(lo, hi, *inclusive)),
            TypeKind::Named { name, args } if args.is_empty() => {
                if let Some(bounds) = self.in_bound(&name.text, |this, ty| this.index(ty)) {
                    return bounds;
                }
                if let Some(bounds) = self.domains.get(&name.text) {
                    return Ok(bounds.clone());
                }
                // `type Cut = V.bounds` names the domain it stands for.
                if let Some((base, alias)) = self.aliases.get(&name.text).cloned() {
                    let outer = std::mem::replace(&mut self.base, base);
                    let bounds = self.index(&alias);
                    self.base = outer;
                    return bounds;
                }
                // `V.cuts` (or `V.bounds`): the gaps around `V`'s positions, one more than it has.
                if let Some(of) = name
                    .text
                    .strip_suffix(".cuts")
                    .or_else(|| name.text.strip_suffix(".bounds"))
                {
                    if let Some((lo, hi)) = self.domains.get(of) {
                        return Ok((lo.clone(), binary(BinOp::Add, hi.clone(), int(1))));
                    }
                }
                // An enum of plain variants is a domain of that many positions.
                if let Some((_, Record::Enum(variants))) = self.records.get(&name.text) {
                    if variants.iter().all(|(_, fields)| fields.is_empty()) {
                        return Ok((int(0), int(variants.len() as i128 - 1)));
                    }
                }
                Err(self.error(
                    ty.span,
                    format!("`{}` is not a domain declared above", name.text),
                ))
            }
            // `Pair<V>`, `Subset<B>`: a std `impl Index` type over a domain,
            // its count as std's `impl Index` states it.
            TypeKind::Named { name, args } if args.len() == 1 => {
                let (lo, hi) = self.index(&args[0])?;
                let n = binary(BinOp::Add, binary(BinOp::Sub, hi, lo), int(1));
                let count = std_index_count(&name.text, &n).ok_or_else(|| {
                    self.error(ty.span, "an array is indexed by a domain or a range".into())
                })?;
                Ok((int(0), binary(BinOp::Sub, count, int(1))))
            }
            _ => Err(self.error(ty.span, "an array is indexed by a domain or a range".into())),
        }
    }

    /// The bounds of each dimension an index type spans: a product domain
    /// `(Row, Col)` spans two, anything else one.
    pub(super) fn dims(&mut self, ty: &Type) -> Result<Vec<(Expr, Expr)>, SpecError> {
        if let TypeKind::Named { name, args } = &ty.kind {
            if args.is_empty() {
                if let Some(dims) = self.in_bound(&name.text, |this, ty| this.dims(ty)) {
                    return dims;
                }
            }
            if let (Some(parts), true) = (self.products.get(&name.text).cloned(), args.is_empty()) {
                let mut dims = Vec::new();
                for part in parts {
                    dims.extend(self.dims(&part)?);
                }
                return Ok(dims);
            }
        }
        Ok(vec![self.index(ty)?])
    }

    /// The type of one value of a `List<T>` or a one-dimensional array.
    pub(super) fn cell<'t>(&self, ty: &'t Type) -> Option<&'t Type> {
        match &ty.kind {
            TypeKind::Named { name, args }
                if args.len() == 1 && checked_type(&name.text, 1).held_elem().is_some() =>
            {
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
            // A view of `S` reads as `S` does.
            TypeKind::Named { name, args }
                if args.len() == 1 && checked_type(&name.text, 1).view_source().is_some() =>
            {
                self.shape(&args[0])
            }
            TypeKind::Optional(inner) => {
                // Without `read(none: x)`, a `?T` is `None` only once the
                // input has ended (R10.2.10): nothing then, a `T` before.
                let Some(none) = self.none else {
                    return match self.ended {
                        true => Ok(Shape::tuple(Vec::new())),
                        false => self.shape(inner),
                    };
                };
                match self.shape(inner)? {
                    inner @ Shape::Int { .. } => Ok(Shape::Maybe {
                        inner: Box::new(inner),
                        none,
                    }),
                    _ => Err(unsupported(self)),
                }
            }
            // A range open at one end, `0..` or `..=10`, takes `Int`'s there.
            TypeKind::Range { lo, hi, inclusive } => {
                let lo = lo.as_deref().cloned().unwrap_or(int(DEFAULT_INT.0));
                let (hi, inclusive) = match hi {
                    Some(hi) => ((**hi).clone(), *inclusive),
                    None => (int(DEFAULT_INT.1), true),
                };
                let (lo, hi) = Self::inclusive(&self.valued(&lo), &self.valued(&hi), inclusive);
                names_in(&lo, &mut self.spec.sizes);
                names_in(&hi, &mut self.spec.sizes);
                Ok(Shape::Int { lo, hi })
            }
            TypeKind::Named { name, args } if args.is_empty() => {
                if let Some(shape) = self.in_bound(&name.text, |this, ty| this.shape(ty)) {
                    return shape;
                }
                match checked_type(&name.text, 0) {
                    // A whole number is an `F64` too; drawing fractions
                    // waits for a fixed-point value (decision 25.21).
                    Ty::Int | Ty::I128 | Ty::Float => {
                        return Ok(Shape::Int {
                            lo: int(DEFAULT_INT.0),
                            hi: int(DEFAULT_INT.1),
                        })
                    }
                    Ty::U64 => {
                        return Ok(Shape::Int {
                            lo: int(0),
                            hi: int(DEFAULT_INT.1),
                        })
                    }
                    Ty::Char => return Ok(Shape::Char { lo: b'a', hi: b'z' }),
                    Ty::Bool => return Ok(Shape::Bool),
                    Ty::Str => return Ok(Shape::WORD),
                    _ => {}
                }
                if self.std_scope.is_some() {
                    return self
                        .std_type(&name.text, args)
                        .unwrap_or_else(|| Err(unsupported(self)));
                }
                if let Some((lo, hi)) = self.domains.get(&name.text).cloned() {
                    if self.io_base != 0 && self.counted.contains(&name.text) {
                        let shift = |end| binary(BinOp::Add, end, int(self.io_base));
                        return Ok(Shape::Int {
                            lo: shift(lo),
                            hi: shift(hi),
                        });
                    }
                    return Ok(Shape::Int { lo, hi });
                }
                // A position of a product domain is one of each part's.
                if let Some(parts) = self.products.get(&name.text).cloned() {
                    return parts
                        .iter()
                        .map(|part| self.shape(part))
                        .collect::<Result<_, _>>()
                        .map(Shape::tuple);
                }
                if let Some((base, record)) = self.records.get(&name.text).cloned() {
                    let outer = std::mem::replace(&mut self.base, base);
                    let shape = self.record(&name.text, &record);
                    self.base = outer;
                    return shape;
                }
                let Some((base, alias)) = self.aliases.get(&name.text).cloned() else {
                    return self
                        .std_type(&name.text, args)
                        .unwrap_or_else(|| Err(unsupported(self)));
                };
                let outer = std::mem::replace(&mut self.base, base);
                // `@stress(tree) type Tree = [E](u: V, v: V, w: W)` says
                // what the values are, beyond their type.
                let shape = match self.alias_made.get(&name.text).copied() {
                    Some(Made::Perm) => Ok(self.perm(&alias)),
                    Some(Made::Tree) => Ok(self.tree(&alias)),
                    Some(Made::Graph(kind)) => Ok(self.graph(&alias, kind)),
                    None => self.shape(&alias).map(Some),
                };
                self.base = outer;
                shape?.ok_or_else(|| unsupported(self))
            }
            // `List<T>` reads a count, then that many `T`s.
            TypeKind::Named { .. } if self.cell(ty).is_some() => {
                let count = format!("(count at {})", ty.span.shift(self.base).start);
                let elem = self.shape(self.cell(ty).unwrap_or(ty))?;
                self.spec.sizes.insert(count.clone());
                let len = Expr {
                    kind: ExprKind::Name(count.clone()),
                    span: ty.span,
                };
                Ok(Shape::Tuple {
                    parts: vec![
                        Shape::Int {
                            lo: int(0),
                            hi: int(DEFAULT_INT.1),
                        },
                        Shape::Array {
                            len,
                            elem: Box::new(elem),
                            index: None,
                        },
                    ],
                    binds: vec![vec![Some(count)], Vec::new()],
                })
            }
            // `'+' | '-'` or `"add" | "del"`: one of these words.
            TypeKind::LiteralUnion(lits) => {
                let words: Vec<String> = lits
                    .iter()
                    .map(|lit| lit.trim_matches(|c| c == '"' || c == '\'').to_string())
                    .collect();
                if words
                    .iter()
                    .any(|w| w.is_empty() || w.contains(char::is_whitespace))
                {
                    return Err(unsupported(self));
                }
                Ok(Shape::Words(words.into()))
            }
            // `Range<T>` reads its two ends `l r` of `T`, `l <= r`.
            TypeKind::Named { name, args } if args.len() == 1 && is_range(&name.text) => {
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
            // `Perm<V>` and `Tree<V>`: a permutation of a domain, and the
            // edges of a tree on it.
            TypeKind::Named { name, args }
                if args.len() == 1
                    && let Some(made @ (Made::Perm | Made::Tree)) =
                        self.std_made(ty, &name.text) =>
            {
                let (lo, hi) = self.index(&args[0])?;
                names_in(&lo, &mut self.spec.sizes);
                names_in(&hi, &mut self.spec.sizes);
                Ok(if made == Made::Perm {
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
                if args.len() == 2
                    && let Some(Made::Graph(kind)) = self.std_made(ty, &name.text) =>
            {
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
            // `[domain S]Char`, `[domain S]Letter`: a word that brings its
            // own length, over the characters its element is.
            TypeKind::OwnDomain { elem, .. } if self.letters(elem).is_some() => {
                let Some((lo, hi)) = self.letters(elem) else {
                    return Err(unsupported(self));
                };
                Ok(Shape::Str {
                    alphabet: ((lo, hi) != (b'a', b'z')).then(|| (lo..=hi).collect()),
                    len: None,
                })
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
            TypeKind::Named { name, args } => self
                .std_type(&name.text, args)
                .unwrap_or_else(|| Err(unsupported(self))),
            TypeKind::Record(fields) => fields
                .iter()
                .map(|(_, field)| self.shape(field))
                .collect::<Result<_, _>>()
                .map(Shape::tuple),
            TypeKind::Array { index, elem } => {
                let mut shape = self.shape(elem)?;
                // `[Row, Col]T` is rows of columns.
                let mut dims = Vec::new();
                for dimension in index {
                    dims.extend(self.dims(dimension)?);
                }
                for (lo, hi) in dims.into_iter().rev() {
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

    /// What `@stress(..)` on the std declaration the written type `ty`
    /// names says it makes: the declaration the checker resolved `ty` to,
    /// so a user's own `Perm` is not std's, or, inside a std type, the
    /// module's own `name`.
    pub(super) fn std_made(&self, ty: &Type, name: &str) -> Option<Made> {
        let (module, name) = match self.std_scope {
            Some(module) => (module, name),
            None => {
                let facts = self.read_types.facts.as_ref()?;
                let decl = facts.imported_type(ty.span.shift(self.base))?;
                (decl.module.strip_prefix("std.")?, decl.name.as_str())
            }
        };
        Made::of(&std_decl(module, name)?.2.decorators)
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
        let Some(elem) = pass(parts, binds) else {
            return Ok(None);
        };
        names_in(&lo, &mut self.spec.sizes);
        names_in(&hi, &mut self.spec.sizes);
        Ok(Some(Shape::Array {
            len: binary(BinOp::Add, binary(BinOp::Sub, hi, lo.clone()), int(1)),
            elem: Box::new(elem),
            index: index.map(|name| (name, lo)),
        }))
    }

    /// `for x in read<[n]T>():` whose body reads too: the loop reads the
    /// whole array before its first pass, so the array, then one row of
    /// what a pass reads for each of its elements.
    pub(super) fn walked(&mut self, stmt: &Stmt) -> Result<Option<Vec<Shape>>, SpecError> {
        let StmtKind::For {
            pat, iter, body, ..
        } = &stmt.kind
        else {
            return Ok(None);
        };
        let mut shapes = Vec::new();
        if !self.reads_in(iter) || !self.inline(iter, &mut shapes)? {
            return Ok(None);
        }
        let Some(Shape::Array { len, .. }) = shapes.last() else {
            return Ok(None);
        };
        let len = len.clone();
        let mut inside = HashSet::new();
        bind_names(pat, &mut inside);
        let (mut parts, mut binds) = (Vec::new(), Vec::new());
        if !self.rows(&body.stmts, &mut inside, &mut parts, &mut binds)? {
            return Ok(None);
        }
        let Some(elem) = pass(parts, binds) else {
            return Ok(None);
        };
        shapes.push(Shape::Array {
            len,
            elem: Box::new(elem),
            index: None,
        });
        Ok(Some(shapes))
    }

    /// A `while true:` loop whose body reads a row and leaves at `if a ==
    /// 0 and b == 0: break` once the reads before it bind `a` and `b`:
    /// rows of what the body reads, ended by those values.
    pub(super) fn until(&mut self, stmt: &Stmt) -> Result<Option<Shape>, SpecError> {
        self.until_exit(stmt, false)
    }

    /// `for x in f():` over a fn of [`Builder::streams`], its own body
    /// reading nothing: what the stream's loop reads, until its sentinel
    /// `return`s.
    pub(super) fn stream(&mut self, stmt: &Stmt) -> Result<Option<Shape>, SpecError> {
        let StmtKind::For { iter, body, .. } = &stmt.kind else {
            return Ok(None);
        };
        let ExprKind::Call { callee, args } = &iter.kind else {
            return Ok(None);
        };
        let ExprKind::Name(name) = &callee.kind else {
            return Ok(None);
        };
        let Some(&(base, looped)) = self.streams.get(name.as_str()) else {
            return Ok(None);
        };
        if !args.is_empty() || body.stmts.iter().any(|stmt| self.reads(stmt)) {
            return Ok(None);
        }
        let outer = std::mem::replace(&mut self.base, base);
        let shape = self.until_exit(looped, true);
        self.base = outer;
        shape
    }

    /// [`Builder::until`], where `returns` lets a `return` leave as a
    /// `break` does.
    fn until_exit(&mut self, stmt: &Stmt, returns: bool) -> Result<Option<Shape>, SpecError> {
        if let StmtKind::While {
            cond:
                Cond::Let {
                    pat,
                    value,
                    and: None,
                },
            body,
            else_body: None,
        } = &stmt.kind
        {
            return self.until_end(pat, value, &body.stmts);
        }
        let StmtKind::While {
            cond: Cond::Expr(cond),
            body,
            else_body: None,
        } = &stmt.kind
        else {
            return Ok(None);
        };
        if !matches!(cond.kind, ExprKind::Bool(true)) {
            return Ok(None);
        }
        let stmts = body.stmts.as_slice();
        if let [Stmt {
            kind:
                StmtKind::Expr(Expr {
                    kind: ExprKind::Match { scrutinee, arms },
                    ..
                }),
            ..
        }] = stmts
        {
            return self.until_variant(scrutinee, arms);
        }
        let Some((at, tests)) = stmts
            .iter()
            .enumerate()
            .find_map(|(at, stmt)| sentinel(stmt, returns).map(|tests| (at, tests)))
        else {
            return Ok(None);
        };
        let mut inside = HashSet::new();
        let (mut parts, mut binds) = (Vec::new(), Vec::new());
        if !self.rows(&stmts[..at], &mut inside, &mut parts, &mut binds)? {
            return Ok(None);
        }
        let stop: Option<Vec<i128>> = binds
            .iter()
            .flatten()
            .take(tests.len())
            .map(|name| {
                let name = name.as_ref()?;
                tests.iter().find(|(test, _)| test == name).map(|(_, v)| *v)
            })
            .collect();
        let Some(stop) = stop.filter(|stop| stop.len() == tests.len()) else {
            return Ok(None);
        };
        if !self.rows(&stmts[at + 1..], &mut inside, &mut parts, &mut binds)? {
            return Ok(None);
        }
        Ok(Some(Shape::Until {
            elem: Box::new(Shape::Tuple { parts, binds }),
            stop,
        }))
    }

    /// `while let x = read<?T>():`: rows, each a `T` and the body's
    /// reads, until the input ends (R10.2.10).
    fn until_end(
        &mut self,
        pat: &Pattern,
        value: &Expr,
        stmts: &[Stmt],
    ) -> Result<Option<Shape>, SpecError> {
        let Some((ty, None)) = typed_read(None, value) else {
            return Ok(None);
        };
        let TypeKind::Optional(inner) = &ty.kind else {
            return Ok(None);
        };
        let first = self.shape(inner)?;
        let (mut parts, mut binds) = (Vec::new(), vec![Self::names(pat, &first)]);
        parts.push(first);
        if !self.rows(stmts, &mut HashSet::new(), &mut parts, &mut binds)? {
            return Ok(None);
        }
        self.ended = true;
        Ok(Some(Shape::Until {
            elem: Box::new(Shape::Tuple { parts, binds }),
            stop: Vec::new(),
        }))
    }

    /// `match read<Cmd>():` whose `Stop => break` arm leaves the loop:
    /// commands until a `Stop`.
    fn until_variant(
        &mut self,
        scrutinee: &Expr,
        arms: &[cpl_syntax::ast::Arm],
    ) -> Result<Option<Shape>, SpecError> {
        let Some((ty, none)) = typed_read(None, scrutinee) else {
            return Ok(None);
        };
        let TypeKind::Named {
            name: enum_name, ..
        } = &ty.kind
        else {
            return Ok(None);
        };
        let stop = arms.iter().find_map(|arm| {
            let variant = match &arm.pat.kind {
                PatternKind::Name(name) => name.as_str(),
                PatternKind::Variant { path, args } if args.is_empty() => {
                    path.last()?.text.as_str()
                }
                _ => return None,
            };
            let leaves = matches!(
                arm.body.stmts.as_slice(),
                [Stmt {
                    kind: StmtKind::Break(None),
                    ..
                }]
            );
            let (of, at) = self.variants.get(variant)?;
            (leaves && arm.guard.is_none() && *of == enum_name.text).then_some(*at)
        });
        let Some(at) = stop else {
            return Ok(None);
        };
        let shape = self.shape_written(&ty, none)?;
        let Shape::Choice { arms: choices, .. } = &shape else {
            return Ok(None);
        };
        let Some([tag]) = choices.get(at).and_then(|arm| arm.tags.as_deref()) else {
            return Ok(None);
        };
        let stop = vec![*tag];
        Ok(Some(Shape::Until {
            elem: Box::new(shape),
            stop,
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
        if let ExprKind::Call { callee, args } = &expr.kind {
            if let ExprKind::Name(name) = &callee.kind {
                if let Some(read) = self.called(name, args, expr.span, out)? {
                    return Ok(read);
                }
            }
        }
        if let Some(ty) = self.read_as(expr) {
            out.push(self.shape(&ty)?);
            return Ok(true);
        }
        if let ExprKind::Call { callee, args } = &expr.kind {
            if let ExprKind::Name(name) = &callee.kind {
                if let Some((base, types)) = self.values.get(name).cloned() {
                    for arg in args {
                        if !self.inline(&arg.value, out)? {
                            return Ok(false);
                        }
                    }
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
            let Some(elem) = pass(inner, Vec::new()) else {
                return Ok(true);
            };
            names_in(&len, &mut this.spec.sizes);
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
            } if let Some((body, _)) = self.skipped_cell(domains, params, body)
                && let Some((lo, hi)) = self.cells(&domains[0]) =>
            {
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
                // Its iterator runs once, before the first element:
                // `x * 2 for x in read<[n]Int>()` reads that array.
                [clause] if clause.conds.is_empty() && !self.reads_in(body) => {
                    self.inline(&clause.iter, out)
                }
                _ => Ok(!self.reads_in(expr)),
            },
            _ => Ok(!self.reads_in(expr)),
        }
    }

    /// What a call of `name` reads, when `name` is `read`, `ask` or a
    /// fn that reads for its caller: its arguments' reads, then its own.
    /// `None` for any other fn.
    fn called(
        &mut self,
        name: &str,
        args: &[cpl_syntax::ast::Arg],
        at: Span,
        out: &mut Vec<Shape>,
    ) -> Result<Option<bool>, SpecError> {
        let builtin = matches!(name, "read" | "ask");
        let body = self
            .bodies
            .get_key_value(name)
            .map(|(&name, &body)| (name, body));
        if self.streams.contains_key(name) || (name == "read" && !args.is_empty()) {
            return Ok(Some(false));
        }
        let block = self
            .blocks
            .get_key_value(name)
            .filter(|_| !self.values.contains_key(name))
            .map(|(&name, (base, body, params))| (name, (*base, *body, params.clone())));
        if !builtin && body.is_none() && block.is_none() {
            return Ok(None);
        }
        if let Some(name) = body
            .map(|(name, _)| name)
            .or(block.as_ref().map(|(name, _)| *name))
        {
            if self.inlining.contains(&name) {
                return Err(self.error(
                    at,
                    format!("`{name}` reads input and calls itself, so how much it reads depends on values a generated input can't know; read the input at top level and pass the values in"),
                ));
            }
        }
        for arg in args {
            if !self.inline(&arg.value, out)? {
                return Ok(Some(false));
            }
        }
        if let Some((name, (base, body, params))) = block {
            self.inlining.push(name);
            let outer = std::mem::replace(&mut self.base, base);
            // What runs after the last read (its `return`) reads nothing.
            let last = body.iter().rposition(|stmt| self.reads(stmt));
            let body = &body[..last.map_or(0, |last| last + 1)];
            let (mut parts, mut binds) = (Vec::new(), Vec::new());
            let read = self.rows(
                body,
                &mut params.into_iter().collect(),
                &mut parts,
                &mut binds,
            );
            self.base = outer;
            self.inlining.pop();
            if !read? {
                return Ok(Some(false));
            }
            out.extend(pass(parts, binds));
            return Ok(Some(true));
        }
        if let Some((name, (base, body))) = body {
            self.inlining.push(name);
            let outer = std::mem::replace(&mut self.base, base);
            let read = self.inline(body, out);
            self.base = outer;
            self.inlining.pop();
            return read.map(Some);
        }
        let Some(snippet) = self.read_types.reads.get(&at.shift(self.base)) else {
            return Ok(Some(false));
        };
        let parsed = cpl_syntax::parse_file(snippet);
        let typed = parsed
            .items
            .first()
            .and_then(|(base, item)| match item.stmts.first() {
                Some(Stmt {
                    kind: StmtKind::Let { ty: Some(ty), .. },
                    ..
                }) if parsed.diagnostics.is_empty() => Some((*base, ty.clone())),
                _ => None,
            });
        let Some((base, ty)) = typed else {
            return Ok(Some(false));
        };
        let (src, outer) = (
            std::mem::replace(&mut self.src, snippet),
            std::mem::replace(&mut self.base, base),
        );
        let shape = self.shape(&ty);
        (self.src, self.base) = (src, outer);
        out.push(shape?);
        Ok(Some(true))
    }

    pub(super) fn reads_in(&self, expr: &Expr) -> bool {
        self.reads_code(&self.src[span_range(expr.span.shift(self.base))])
    }

    /// True when `code` reads: it calls `read`, or a fn of [`values`].
    pub(super) fn reads_code(&self, code: &str) -> bool {
        reads_inside(code) || {
            let tokens = cpl_syntax::lex(code).tokens;
            (0..tokens.len().saturating_sub(1)).any(|at| {
                let name = &code[span_range(tokens[at].span)];
                (at == 0 || tokens[at - 1].kind != cpl_syntax::TokenKind::Dot)
                    && tokens[at].kind == cpl_syntax::TokenKind::Ident
                    && tokens[at + 1].kind == cpl_syntax::TokenKind::LParen
                    && (name == "ask"
                        || self.values.contains_key(name)
                        || self.bodies.contains_key(name)
                        || self.blocks.contains_key(name)
                        || self.streams.contains_key(name))
            })
        }
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

    /// True when `stmt` is an `impl` whose `fn read(mut input: Input)` reads
    /// its `input`: `input.read()` somewhere in it.
    pub(super) fn reads_input(&self, stmt: &Stmt) -> bool {
        let StmtKind::Impl { body, .. } = &stmt.kind else {
            return false;
        };
        body.stmts.iter().any(|item| match &item.kind {
            StmtKind::Fn(decl) if decl.name.text == "read" => {
                decl.params.first().is_some_and(|input| {
                    let code = &self.src[span_range(item.span.shift(self.base))];
                    let tokens = cpl_syntax::lex(code).tokens;
                    let text = |at: usize| &code[span_range(tokens[at].span)];
                    (2..tokens.len()).any(|at| {
                        text(at - 2) == input.name.text
                            && tokens[at - 1].kind == cpl_syntax::TokenKind::Dot
                            && text(at) == "read"
                    })
                })
            }
            _ => false,
        })
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
                } if let Some((ty, none)) = typed_read(ty.as_ref(), init) => {
                    let mut used = HashSet::new();
                    Sizes(&mut used).of_type(&ty);
                    type_names(&ty, &mut used);
                    if !used.is_disjoint(inside) {
                        return Ok(false);
                    }
                    let shape = self.shape_read(&ty, none, init)?;
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
                StmtKind::Domain { name, def, .. } => {
                    self.note_char_domain(&name.text, def);
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
                } if let Some((base, ty)) = self.elem_read(target, value) => {
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

    /// The shape of `ty` read with `none` written as `none`.
    /// The shape `read<T>(base: k)` (or a read under `with io(base: k)`)
    /// writes: counted domains shifted by `k`.
    pub(super) fn shape_read(
        &mut self,
        ty: &Type,
        none: Option<i128>,
        init: &Expr,
    ) -> Result<Shape, SpecError> {
        let outer = self.io_base;
        self.io_base = read_base(init).unwrap_or(outer);
        let shape = self.shape_written(ty, none);
        self.io_base = outer;
        shape
    }

    /// An impl's `fn read(mut input: Input)` for `Cell` (what `read<Cell>()`
    /// calls through `T.read(..)`) reads a `Cell` one field per
    /// `let name: Type = input.read()` line (or `read()`, in a `fn read()` of
    /// the old pipeline). `None` when it reads in any other way.
    pub(super) fn read_impl(
        &mut self,
        stmt: &Stmt,
    ) -> Result<Option<(String, Vec<Shape>)>, SpecError> {
        let StmtKind::Impl {
            trait_name: Some(trait_name),
            target,
            body,
            ..
        } = &stmt.kind
        else {
            return Ok(None);
        };
        let (TypeKind::Named { .. }, TypeKind::Named { name: target, args }) =
            (&trait_name.kind, &target.kind)
        else {
            return Ok(None);
        };
        if !args.is_empty() {
            return Ok(None);
        }
        let read = body.stmts.iter().find_map(|stmt| match &stmt.kind {
            StmtKind::Fn(decl) if decl.name.text == "read" && decl.params.len() <= 1 => Some(decl),
            _ => None,
        });
        let Some(cpl_syntax::ast::FnBody::Block(block)) = read.map(|decl| &decl.body) else {
            return Ok(None);
        };
        let input = read
            .and_then(|decl| decl.params.first())
            .map(|p| p.name.text.as_str());
        let mut parts = Vec::new();
        for line in &block.stmts {
            if let StmtKind::Let {
                ty,
                init: Some(init),
                ..
            } = &line.kind
            {
                let init = &input
                    .and_then(|input| plain_read(init, input))
                    .unwrap_or_else(|| init.clone());
                if let Some((ty, none)) = typed_read(ty.as_ref(), init) {
                    parts.push(self.shape_read(&ty, none, init)?);
                    continue;
                }
            }
            if self.reads(line) {
                return Ok(None);
            }
        }
        Ok(Some((target.text.clone(), parts)))
    }

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

    /// `e` with each std generic it names (`P` in `Mod<P>(pub v: 0..P)`)
    /// replaced by the value it stands for.
    fn valued(&self, e: &Expr) -> Expr {
        match &e.kind {
            ExprKind::Name(name) => match self.bound.get(name).map(|b| &b.ty.kind) {
                Some(TypeKind::Value(p)) => (**p).clone(),
                Some(TypeKind::Named { name, args }) if args.is_empty() => Expr {
                    kind: ExprKind::Name(name.text.clone()),
                    span: name.span,
                },
                _ => e.clone(),
            },
            ExprKind::Binary { op, lhs, rhs, .. } => {
                binary(*op, self.valued(lhs), self.valued(rhs))
            }
            _ => e.clone(),
        }
    }

    /// Runs `f` on what the std generic `name` stands for, in the text and
    /// scope it is written in.
    fn in_bound<R>(&mut self, name: &str, f: impl FnOnce(&mut Self, &Type) -> R) -> Option<R> {
        let bound = self.bound.get(name)?.clone();
        let saved = (
            std::mem::replace(&mut self.src, bound.src),
            std::mem::replace(&mut self.base, bound.base),
            std::mem::replace(&mut self.std_scope, bound.scope),
            std::mem::replace(&mut self.bound, bound.outer.clone()),
        );
        let out = f(self, &bound.ty);
        (self.src, self.base, self.std_scope, self.bound) = saved;
        Some(out)
    }

    /// A std type as `read()` reads it (`cpl_ir`'s `struct_spec`): a
    /// one-field struct through its one-parameter `new`, else its fields
    /// in order when they are all `pub`; an alias as what it names. `None`
    /// when std declares no such type.
    pub(super) fn std_type(
        &mut self,
        name: &str,
        args: &[Type],
    ) -> Option<Result<Shape, SpecError>> {
        let module = self
            .std_scope
            .filter(|m| std_decl(m, name).is_some())
            .or_else(|| cpl_facade::Std.module_of(name))?;
        let found = std_decl(module, name)?;
        // `Prefix<T>` with one argument names the family member taking one.
        let (module, (text, base, stmt)) = cpl_facade::Std
            .family(name)
            .filter(|_| generic_count(found.2) != Some(args.len()))
            .find_map(|(module, sym)| {
                std_decl(module, sym)
                    .filter(|d| generic_count(d.2) == Some(args.len()))
                    .map(|d| (module, d))
            })
            .unwrap_or((module, found));
        let mut tagged = None;
        let (generics, defaults, types, fields): (_, Vec<Option<Type>>, Vec<Type>, bool) =
            match &stmt.kind {
                StmtKind::Struct(decl) => {
                    let method = |want: &str, params: usize| {
                        decl.methods.iter().find_map(|m| match &m.kind {
                            StmtKind::Fn(f)
                                if f.name.text == want
                                    && f.params.len() == params
                                    && f.params.iter().all(|p| p.default.is_none()) =>
                            {
                                Some(f)
                            }
                            _ => None,
                        })
                    };
                    if method("read", 0).is_some() {
                        return None;
                    }
                    let new = method("new", 1).and_then(|f| f.params[0].ty.clone());
                    // A public field says which values the struct holds
                    // (`Mod<P>(pub v: 0..P)`), tighter than what `new`
                    // reduces into one.
                    let (types, fields) = match new {
                        Some(_) if decl.fields.len() == 1 && decl.fields[0].public => {
                            (vec![decl.fields[0].ty.clone()], false)
                        }
                        Some(param) if decl.fields.len() == 1 => (vec![param], false),
                        _ if decl.fields.iter().all(|f| f.public) => {
                            (decl.fields.iter().map(|f| f.ty.clone()).collect(), true)
                        }
                        _ => return None,
                    };
                    (&decl.generics, decl.generic_defaults.clone(), types, fields)
                }
                StmtKind::TypeAlias { generics, ty, .. } => (
                    generics,
                    vec![None; generics.len()],
                    vec![ty.clone()],
                    false,
                ),
                StmtKind::Enum(decl) => {
                    tagged = Some(Record::tagged(stmt, decl)?);
                    (
                        &decl.generics,
                        vec![None; decl.generics.len()],
                        Vec::new(),
                        true,
                    )
                }
                _ => return None,
            };
        let mut bound = HashMap::new();
        for (at, generic) in generics.iter().enumerate() {
            let here = match args.get(at) {
                Some(arg) => Bound {
                    src: self.src,
                    base: self.base,
                    ty: arg.clone(),
                    scope: self.std_scope,
                    outer: self.bound.clone(),
                },
                None => match defaults.get(at).cloned().flatten() {
                    Some(ty) => Bound {
                        src: text,
                        base,
                        ty,
                        scope: Some(module),
                        outer: Rc::default(),
                    },
                    None => continue,
                },
            };
            bound.insert(generic.text.clone(), here);
        }
        let saved = (
            std::mem::replace(&mut self.src, text),
            std::mem::replace(&mut self.base, base),
            self.std_scope.replace(module),
            std::mem::replace(&mut self.bound, Rc::new(bound)),
        );
        let parts: Result<Vec<Shape>, SpecError> = match &tagged {
            Some(record) => self.record(name, record).map(|shape| vec![shape]),
            None => types.iter().map(|ty| self.shape(ty)).collect(),
        };
        (self.src, self.base, self.std_scope, self.bound) = saved;
        let fields = fields && tagged.is_none();
        Some(parts.map(|mut parts| match (fields, parts.len()) {
            (false, 1) => parts.remove(0),
            _ => Shape::tuple(parts),
        }))
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
                        lo: letters.iter().copied().min().unwrap_or(b'a'),
                        hi: letters.iter().copied().max().unwrap_or(b'a'),
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

/// The names and values `if a == 0 and b == -1: break` leaves a loop at.
fn sentinel(stmt: &Stmt, returns: bool) -> Option<Vec<(String, i128)>> {
    let StmtKind::Expr(Expr {
        kind: ExprKind::If {
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
    match then.stmts.as_slice() {
        [Stmt {
            kind: StmtKind::Break(None),
            ..
        }] => {}
        [Stmt {
            kind: StmtKind::Return(None),
            ..
        }] if returns => {}
        _ => return None,
    }
    fn tests(cond: &Expr, out: &mut Vec<(String, i128)>) -> Option<()> {
        match &cond.kind {
            ExprKind::Binary {
                op: BinOp::And,
                lhs,
                rhs,
                ..
            } => {
                tests(lhs, out)?;
                tests(rhs, out)
            }
            ExprKind::Compare { first, rest } => match (&first.kind, rest.as_slice()) {
                (ExprKind::Name(name), [(CmpOp::Eq, value)]) => {
                    let value = match &value.kind {
                        ExprKind::Int(v) => *v,
                        ExprKind::Unary {
                            op: UnOp::Neg,
                            expr,
                        } => match expr.kind {
                            ExprKind::Int(v) => -v,
                            _ => return None,
                        },
                        _ => return None,
                    };
                    out.push((name.clone(), value));
                    Some(())
                }
                _ => None,
            },
            _ => None,
        }
    }
    let mut out = Vec::new();
    tests(cond, &mut out)?;
    Some(out)
}

/// The declaration of the std type `name` in `module`, with that module's
/// text and the base its spans are relative to.
pub(super) fn std_decl(module: &str, name: &str) -> Option<(&'static str, u32, &'static Stmt)> {
    let (text, parsed) = std_file(module)?;
    parsed.items.iter().find_map(|(base, item)| {
        item.stmts
            .iter()
            .find(|stmt| match &stmt.kind {
                StmtKind::Struct(decl) => decl.name.text == name,
                StmtKind::TypeAlias { name: alias, .. } => alias.text == name,
                StmtKind::Enum(decl) => decl.name.text == name,
                _ => false,
            })
            .map(|stmt| (text, *base, stmt))
    })
}

/// A std module's text and its parse, parsed once per thread.
/// Whether `name` is the range type: what the checker types `a..b` as.
fn is_range(name: &str) -> bool {
    static RANGE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    let range = RANGE.get_or_init(|| {
        let (src, literal) = ("let r = 0..1\n", Span::new(8, 12));
        let mut db = cpl_facade::Session::default();
        db.set_file("input.cp", src);
        let ty = db.facts("input.cp")?.type_of(literal)?.to_string();
        Some(ty.split('<').next()?.to_string())
    });
    range.as_deref() == Some(name)
}

pub(super) fn std_file(module: &str) -> Option<(&'static str, &'static cpl_syntax::ParsedFile)> {
    thread_local! {
        static PARSED: std::cell::RefCell<HashMap<String, &'static cpl_syntax::ParsedFile>> =
            Default::default();
    }
    let text = cpl_facade::Std.text(module)?;
    let parsed = PARSED.with(|parsed| {
        *parsed
            .borrow_mut()
            .entry(module.to_string())
            .or_insert_with(|| Box::leak(Box::new(cpl_syntax::parse_file(text))))
    });
    Some((text, parsed))
}

/// How many generics a std struct, enum or alias declares.
fn generic_count(stmt: &Stmt) -> Option<usize> {
    match &stmt.kind {
        StmtKind::Struct(decl) => Some(decl.generics.len()),
        StmtKind::Enum(decl) => Some(decl.generics.len()),
        StmtKind::TypeAlias { generics, .. } => Some(generics.len()),
        _ => None,
    }
}

/// What one pass reads: its one part, or its parts as a tuple, `binds`
/// naming each part's fields; `None` when it reads nothing.
fn pass(mut parts: Vec<Shape>, binds: Vec<Vec<Option<String>>>) -> Option<Shape> {
    match parts.len() {
        0 | 1 => parts.pop(),
        _ if binds.iter().all(Vec::is_empty) => Some(Shape::tuple(parts)),
        _ => Some(Shape::Tuple { parts, binds }),
    }
}

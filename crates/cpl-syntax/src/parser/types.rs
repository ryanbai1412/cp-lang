//! Types.

use super::*;

impl<'a> Parser<'a> {
    /// A type; `A -> B` is a fn type, as in `adj: V -> Iter<A>`.
    pub(super) fn ty(&mut self) -> Type {
        let start = self.start();
        let mut first = self.ty_atom();
        if self.at(T::Question) {
            first = self.postfix_optional(first);
        }
        if !self.at(T::Arrow) || matches!(first.kind, TypeKind::Fn { .. }) {
            return first;
        }
        self.bump();
        let ret = Box::new(self.ty());
        Type {
            kind: TypeKind::Fn {
                params: vec![first],
                ret,
            },
            span: self.span_from(start),
        }
    }

    /// `Int?`, Kotlin's and C#'s optional: CP Lang writes `?Int` (R02.4).
    fn postfix_optional(&mut self, inner: Type) -> Type {
        let q = self.span();
        self.bump();
        let span = Span::new(inner.span.start, q.end);
        let mut diag = Diagnostic::error(
            codes::EXPECTED_TYPE,
            span,
            "an optional type puts its `?` first in CP Lang",
        )
        .with_help("write `?T`, as in `?Int`, for a `T` or `None`");
        diag.fixes.push(cpl_diag::Fix {
            title: "move the `?` to the front".into(),
            edits: vec![
                (Span::point(inner.span.start), "?".into()),
                (q, String::new()),
            ],
        });
        self.recovered(diag);
        Type {
            kind: TypeKind::Optional(Box::new(inner)),
            span,
        }
    }

    fn ty_atom(&mut self) -> Type {
        if self.at_ident("dyn") && self.peek_n(1) == T::Ident {
            let span = self.span();
            self.not_in_language(
                span,
                "there is no `dyn`: every trait call is resolved at compile time, with no vtables (R08.2.3)",
                "take a type parameter instead: `fn f<T: Ord>(x: T)`",
            );
            self.bump();
            let named = self.ty_atom();
            return Type {
                kind: TypeKind::Error,
                span: named.span,
            };
        }
        let start = self.start();
        let kind = match self.peek() {
            T::Char | T::Str | T::Int if self.peek_n(1) == T::Pipe => {
                let mut lits = Vec::new();
                while matches!(self.peek(), T::Char | T::Str | T::Int) {
                    lits.push(self.text().to_string());
                    self.bump();
                    if !self.eat(T::Pipe) {
                        break;
                    }
                }
                TypeKind::LiteralUnion(lits)
            }
            T::Question => {
                self.bump();
                TypeKind::Optional(Box::new(self.ty()))
            }
            T::LBracket => {
                self.bump();
                if self.at(T::Domain) {
                    self.bump();
                    let name = self.expect_name("a domain name after `domain`");
                    self.expect(T::RBracket, "`]` after the domain name");
                    TypeKind::OwnDomain {
                        name,
                        elem: Box::new(self.ty()),
                    }
                } else if self.at_ident("_") && self.peek_n(1) == T::RBracket {
                    let span = self.span();
                    self.planned(span, &crate::planned::ANY_LENGTH);
                    self.bump();
                    self.bump();
                    TypeKind::Array {
                        index: vec![Type {
                            kind: TypeKind::Error,
                            span,
                        }],
                        elem: Box::new(self.ty()),
                    }
                } else if self.at_ident("read") && self.peek_n(1) == T::LParen {
                    let read = self.start();
                    self.bump();
                    self.pos = after_group(self.tokens, self.pos);
                    let span = self.span_from(read);
                    self.planned(span, &crate::planned::READ_LENGTH);
                    self.expect(T::RBracket, "`]` after the index");
                    TypeKind::Array {
                        index: vec![Type {
                            kind: TypeKind::Error,
                            span,
                        }],
                        elem: Box::new(self.ty()),
                    }
                } else if self.at(T::LParen)
                    && self
                        .tokens
                        .get(after_group(self.tokens, self.pos))
                        .map(|t| t.kind)
                        == Some(T::RBracket)
                {
                    // `[(A, B)]T`: product index space, identical to `[A, B]T`.
                    self.bump();
                    let index = self.comma_list(T::RParen, "the product index", Self::index_ty);
                    self.expect(T::RBracket, "`]` after the index");
                    TypeKind::Array {
                        index,
                        elem: Box::new(self.ty()),
                    }
                } else {
                    let index = self.comma_list(T::RBracket, "the index list", Self::index_ty);
                    TypeKind::Array {
                        index,
                        elem: Box::new(self.ty()),
                    }
                }
            }
            T::LParen => {
                self.bump();
                if self.eat(T::DotDot) {
                    let base = Box::new(self.ty());
                    let mut removed = Vec::new();
                    if self.at_ident("without") {
                        self.bump();
                        self.expect(T::LParen, "`(` and the removed field names after `without`");
                        removed = self.comma_list(T::RParen, "the removed fields", |p| {
                            p.expect_name("a field name")
                        });
                    }
                    self.expect(T::RParen, "`)`");
                    TypeKind::RowWithout {
                        prefix: Vec::new(),
                        base,
                        removed,
                    }
                } else if self.at(T::Domain) {
                    enum Item {
                        Dom(Name, Span),
                        Named(Name, Type),
                        Pos(Type),
                    }
                    let items = self.comma_list(T::RParen, "the tuple or record type", |p| {
                        if p.at(T::Domain) {
                            let start = p.start();
                            p.bump();
                            let name = p.expect_name("a domain name after `domain`");
                            let span = p.span_from(start);
                            Item::Dom(name, span)
                        } else if p.at(T::Ident) && p.peek_n(1) == T::Colon {
                            let name = p.name();
                            p.bump();
                            Item::Named(name, p.ty())
                        } else {
                            Item::Pos(p.ty())
                        }
                    });
                    let has_named = items.iter().any(|i| matches!(i, Item::Named(..)));
                    if has_named {
                        let fields = items
                            .into_iter()
                            .enumerate()
                            .map(|(idx, item)| match item {
                                Item::Dom(name, span) => (
                                    name,
                                    Type {
                                        kind: TypeKind::NewDomain,
                                        span,
                                    },
                                ),
                                Item::Named(name, ty) => (name, ty),
                                Item::Pos(ty) => (
                                    Name {
                                        text: idx.to_string(),
                                        span: ty.span,
                                    },
                                    ty,
                                ),
                            })
                            .collect();
                        TypeKind::Record(fields)
                    } else {
                        let mut tys: Vec<Type> = items
                            .into_iter()
                            .map(|item| match item {
                                Item::Dom(name, span) => Type {
                                    kind: TypeKind::DomainBinder(name),
                                    span,
                                },
                                Item::Pos(ty) => ty,
                                Item::Named(_, ty) => ty,
                            })
                            .collect();
                        if tys.len() == 1 {
                            return tys.pop().unwrap();
                        }
                        TypeKind::Tuple(tys)
                    }
                } else if self.at(T::Ident) && self.peek_n(1) == T::Colon {
                    let mut fields = Vec::new();
                    let mut open = false;
                    let mut spread: Option<(Box<Type>, Vec<Name>)> = None;
                    while !self.at(T::RParen) && !self.at(T::Eof) {
                        if self.eat(T::DotDot) {
                            if self.at(T::RParen) || self.at(T::Comma) {
                                open = true;
                                self.eat(T::Comma);
                            } else {
                                let base = Box::new(self.ty());
                                let mut removed = Vec::new();
                                if self.at_ident("without") {
                                    self.bump();
                                    self.expect(
                                        T::LParen,
                                        "`(` and the removed field names after `without`",
                                    );
                                    removed =
                                        self.comma_list(T::RParen, "the removed fields", |p| {
                                            p.expect_name("a field name")
                                        });
                                }
                                spread = Some((base, removed));
                            }
                            break;
                        }
                        let name = self.expect_name("a field name");
                        self.expect(T::Colon, "`:` after the field name");
                        fields.push((name, self.ty()));
                        if !self.eat(T::Comma) {
                            break;
                        }
                    }
                    self.expect(T::RParen, "`)`");
                    if let Some((base, removed)) = spread {
                        TypeKind::RowWithout {
                            prefix: fields,
                            base,
                            removed,
                        }
                    } else if open {
                        TypeKind::Row(fields)
                    } else {
                        TypeKind::Record(fields)
                    }
                } else {
                    let mut items = self.comma_list(T::RParen, "the tuple type", Self::ty);
                    if self.eat(T::Arrow) {
                        TypeKind::Fn {
                            params: items,
                            ret: Box::new(self.ty()),
                        }
                    } else if items.len() == 1 {
                        return items.pop().unwrap();
                    } else {
                        TypeKind::Tuple(items)
                    }
                }
            }
            T::Mut => {
                self.bump();
                TypeKind::Mut(Box::new(self.ty()))
            }
            T::Fn => {
                self.bump();
                self.expect(T::LParen, "`(` after `fn` in a fn type");
                let params = self.comma_list(T::RParen, "the fn type", Self::ty);
                let ret = if self.eat(T::Arrow) {
                    self.ty()
                } else {
                    Type {
                        kind: TypeKind::Tuple(Vec::new()),
                        span: Span::point(self.prev_end()),
                    }
                };
                TypeKind::Fn {
                    params,
                    ret: Box::new(ret),
                }
            }
            T::Ident
                if self.text() == "yields"
                    && !matches!(self.peek_n(1), T::DotDot | T::DotDotEq) =>
            {
                self.bump();
                TypeKind::Yields(Box::new(self.ty()))
            }
            T::Ident
                if self.text().starts_with(|c: char| c.is_ascii_uppercase())
                    && !self.range_follows()
                    && self.foreign_type_here().is_none() =>
            {
                let mut name = self.name();
                // `V.cuts`: a domain derived from another.
                while self.at(T::Dot) && self.peek_n(1) == T::Ident {
                    self.bump();
                    let member = self.name();
                    let text = if member.text == "bounds" {
                        "cuts"
                    } else {
                        member.text.as_str()
                    };
                    name.text = format!("{}.{text}", name.text);
                    name.span = name.span.to(member.span);
                }
                if name.text == "Bounds" {
                    name.text = "Cut".into();
                }
                let mut args = if self.at(T::Lt) {
                    self.type_args()
                } else {
                    self.bracket_type_args(&name.text)
                };
                // `Rooted<V>.Pre`: a domain owned by another type.
                while self.at(T::Dot)
                    && self.peek_n(1) == T::Ident
                    && self.tokens[self.pos + 1]
                        .span
                        .text(self.src)
                        .starts_with(|c: char| c.is_ascii_uppercase())
                {
                    self.bump();
                    let member = self.name();
                    name.text = format!("{}.{}", name.text, member.text);
                    name.span = name.span.to(member.span);
                }
                // `Mod(p)`: runtime-modulus modular integer type (same as `Mod<p>`).
                if name.text == "Mod"
                    && args.is_empty()
                    && self.at(T::LParen)
                    && self.span().start == name.span.end
                {
                    self.bump();
                    let arg_start = self.start();
                    let mod_expr = self.expr();
                    let arg_span = self.span_from(arg_start);
                    self.expect(T::RParen, "`)` after the modulus");
                    args.push(Type {
                        kind: TypeKind::Value(Box::new(mod_expr)),
                        span: arg_span,
                    });
                }
                // `Str(len: …)`, `Mint(mod: p)`: a type built from values.
                if args.is_empty() && self.at(T::LParen) && self.span().start == name.span.end {
                    self.pos = after_group(self.tokens, self.pos);
                    let span = self.span_from(name.span.start);
                    let planned = if name.text == "Str" {
                        &crate::planned::BOUNDED_STR
                    } else {
                        &crate::planned::FRESH_TYPE
                    };
                    self.planned(span, planned);
                }
                if name.text == "Str" && self.at_ident("of") && self.peek_n(1) == T::Str {
                    self.bump();
                    let raw = self.text();
                    let alphabet = raw
                        .strip_prefix('"')
                        .and_then(|s| s.strip_suffix('"'))
                        .unwrap_or(raw)
                        .to_string();
                    self.bump();
                    return Type {
                        kind: TypeKind::ConstrainedStr(alphabet),
                        span: self.span_from(start),
                    };
                }
                TypeKind::Named { name, args }
            }
            T::Ident if self.text() == "dyn" && self.peek_n(1) == T::Ident => {
                self.bump();
                let _trait_ty = self.ty_atom();
                let span = self.span_from(start);
                self.error(
                    Diagnostic::error(
                        codes::UNSUPPORTED,
                        span,
                        "CP Lang has no `dyn` trait objects; use a generic type parameter with a trait bound instead",
                    )
                    .with_help("write `fn f<T: Trait>(x: T)`"),
                );
                TypeKind::Error
            }
            // `xs.D`: the domain a value brought with it, as `compress` does.
            T::Ident
                if self.peek_n(1) == T::Dot
                    && self.peek_n(2) == T::Ident
                    && self.tokens[self.pos + 2]
                        .span
                        .text(self.src)
                        .starts_with(|c: char| c.is_ascii_uppercase())
                    && !matches!(self.peek_n(3), T::Dot | T::DotDot | T::DotDotEq | T::LParen) =>
            {
                let mut name = self.name();
                self.bump();
                let member = self.name();
                name.text = format!("{}.{}", name.text, member.text);
                name.span = name.span.to(member.span);
                TypeKind::Named {
                    name,
                    args: Vec::new(),
                }
            }
            T::DotDot | T::DotDotEq => {
                // One-sided upper-bounded range type: `..n`, `..=8`.
                let inclusive = self.at(T::DotDotEq);
                self.range_op();
                let hi = self.range_bound();
                TypeKind::Range {
                    lo: None,
                    hi: Some(Box::new(hi)),
                    inclusive,
                }
            }
            // `int`, `string`, `vector<Int>`: another language's type name,
            // read as ours so the declaration and its uses check.
            T::Ident
                if self.foreign_type_here().is_some()
                    && !matches!(self.peek_n(1), T::DotDot | T::DotDotEq | T::Dot) =>
            {
                let ours = self.foreign_type_here().unwrap_or_default();
                let mut name = self.name();
                let help = if name.text.starts_with(|c: char| c.is_ascii_uppercase()) {
                    "CP Lang's growable list is `List<T>`"
                } else {
                    "built-in types are capitalized: `Int`, `Str`, `List<T>`"
                };
                self.error(
                    Diagnostic::error(
                        codes::EXPECTED_TYPE,
                        name.span,
                        format!("CP Lang spells the type `{ours}`, not `{}`", name.text),
                    )
                    .with_help(help)
                    .with_fix(format!("change to `{ours}`"), name.span, ours),
                );
                name.text = ours.to_string();
                let args = if self.at(T::Lt) {
                    self.type_args()
                } else {
                    self.bracket_type_args(ours)
                };
                TypeKind::Named { name, args }
            }
            T::Ident | T::Int | T::Minus | T::Float => {
                // A range type: `0..=1e9`, `1..=n`, or one-sided `0..`, `2..`.
                let lo = self.range_bound();
                let inclusive = self.at(T::DotDotEq);
                if !inclusive && !self.at(T::DotDot) {
                    let span = lo.span;
                    self.error(
                        Diagnostic::error(codes::EXPECTED_TYPE, span, "expected a type").with_help(
                            "types are capitalized (`Int`, `V`), or ranges such as `0..=1e9`",
                        ),
                    );
                    TypeKind::Error
                } else {
                    self.range_op();
                    let hi = if inclusive
                        || matches!(
                            self.peek(),
                            T::Ident | T::Int | T::Minus | T::Float | T::LParen | T::SelfKw
                        ) {
                        Some(Box::new(self.range_bound()))
                    } else {
                        None
                    };
                    TypeKind::Range {
                        lo: Some(Box::new(lo)),
                        hi,
                        inclusive,
                    }
                }
            }
            // `xs: &List<Int>`: a parameter is already lent (R4), so drop `&`.
            T::Amp => {
                let amp = self.span();
                self.recovered(
                    Diagnostic::error(codes::EXPECTED_TYPE, amp, "CP Lang has no `&` types")
                        .with_help("a parameter is lent without a copy: write `xs: List<Int>`")
                        .with_fix("remove the `&`", amp, ""),
                );
                self.bump();
                return self.ty();
            }
            // `let x: = 1`: the annotation is empty, so drop its `:`.
            T::Eq if self.prev_kind() == Some(T::Colon) => {
                let colon = self.tokens[self.pos - 1].span;
                let found = self.peek().describe();
                let message = format!("expected a type after `:`, found {found}");
                let diag = Diagnostic::error(codes::EXPECTED_TYPE, self.error_span(), message)
                    .with_fix("remove the `:`", colon, "");
                self.error(diag);
                TypeKind::Error
            }
            _ => {
                let after = self.prev_kind().map_or("here", |k| k.describe());
                self.error_expected(codes::EXPECTED_TYPE, &format!("a type after {after}"), None);
                TypeKind::Error
            }
        };
        Type {
            kind,
            span: self.span_from(start),
        }
    }

    /// One entry of `[...]T`: a domain, or a count as in `[n]Int`, which
    /// is the range `0..n`.
    pub(super) fn index_ty(&mut self) -> Type {
        // A count is an `Int` expression, `n` or `n - 1`; a range has
        // its `..`, and a domain starts with a capital.
        let entry: Vec<T> = self.tokens[self.pos..]
            .iter()
            .take_while(|t| !matches!(t.kind, T::RBracket | T::Comma | T::Newline | T::Eof))
            .map(|t| t.kind)
            .collect();
        let mut depth = 0i32;
        let mut has_group = false;
        let mut has_arith = false;
        for t in &entry {
            match t {
                T::LParen | T::LBracket => {
                    has_group = true;
                    depth += 1;
                }
                T::RParen => depth -= 1,
                T::Plus | T::Minus | T::Star | T::Slash | T::Shl if depth == 0 => has_arith = true,
                _ => {}
            }
        }
        let has_range = entry.iter().any(|t| matches!(t, T::DotDot | T::DotDotEq));
        let has_range_or_group = has_range || (has_group && !has_arith);
        let counted = !has_range_or_group
            && (has_arith
                || (matches!(self.peek(), T::Ident | T::Int)
                    && !self.text().starts_with(|c: char| c.is_ascii_uppercase())));
        if !counted {
            return self.ty();
        }
        let hi = self.primary_for_count();
        let span = hi.span;
        let lo = Expr {
            kind: ExprKind::Int(0),
            span: Span::point(span.start),
        };
        Type {
            kind: TypeKind::Range {
                lo: Some(Box::new(lo)),
                hi: Some(Box::new(hi)),
                inclusive: false,
            },
            span,
        }
    }

    fn primary_for_count(&mut self) -> Expr {
        self.range_bound()
    }

    /// A range bound in a type: arithmetic and `<<`, but not `>>`, which
    /// closes nested type arguments.
    fn range_bound(&mut self) -> Expr {
        let mut lhs = self.additive();
        while self.at(T::Shl) {
            let start = lhs.span.start;
            lhs = self.binary(start, BinOp::Shl, lhs, Self::additive);
        }
        lhs
    }

    /// True when a capitalized name (possibly with `.member` access, such as
    /// `Self.min..=Self.max`) starts a range bound and not a type name.
    fn range_follows(&self) -> bool {
        let mut i = 1;
        while self.peek_n(i) == T::Dot && self.peek_n(i + 1) == T::Ident {
            i += 2;
        }
        matches!(self.peek_n(i), T::DotDot | T::DotDotEq)
    }

    /// Parses `<T, U>`. A closing `>>` counts as two closers.
    pub(super) fn type_args(&mut self) -> Vec<Type> {
        self.bump();
        let mut args = Vec::new();
        loop {
            if self.at_type_args_end() || self.at_line_end() {
                break;
            }
            // `Mod<7>`, `Mod<1e9 + 7>`, `Fixed<d>`: a value argument.
            let value = (matches!(self.peek(), T::Int | T::Minus)
                || (self.at(T::Ident)
                    && !self.text().starts_with(|c: char| c.is_ascii_uppercase())))
                && matches!(
                    self.peek_n(1),
                    T::Comma | T::Gt | T::Shr | T::Plus | T::Minus | T::Star | T::StarStar
                );
            if value {
                let expr = self.additive();
                args.push(Type {
                    span: expr.span,
                    kind: TypeKind::Value(Box::new(expr)),
                });
            } else {
                args.push(self.ty());
            }
            if !self.eat(T::Comma) {
                break;
            }
        }
        self.close_angle("`>` to close the type arguments");
        args
    }

    /// Eats one `>`, which may be half of a `>>` that closes two lists.
    fn close_angle(&mut self, expected: &str) {
        if self.at(T::Gt) {
            self.bump();
        } else if self.at(T::Shr) {
            if self.half_shr {
                self.bump();
            }
            self.half_shr = !self.half_shr;
        } else {
            self.error_expected(codes::EXPECTED_TOKEN, expected, Some(">"));
        }
    }

    fn at_type_args_end(&self) -> bool {
        matches!(self.peek(), T::Gt | T::Shr)
    }

    /// Parses `<D, T: Ord + Add, W = (), const N: Int>` after a fn or type
    /// name: the parameter names, optional defaults, and bounds.
    pub(super) fn generic_params_with_defaults(
        &mut self,
    ) -> (Vec<Name>, Vec<Option<Type>>, Vec<Bound>) {
        let mut defaults = Vec::new();
        let mut bounds = Vec::new();
        if !self.eat(T::Lt) {
            return (Vec::new(), defaults, bounds);
        }
        let mut names = Vec::new();
        while !self.at_type_args_end() && !self.at_line_end() {
            // `const N: Int` is a value parameter; its type is not a bound.
            let value = self.eat(T::Const);
            // `<domain I>` marks a parameter that is an index space; it is still a type.
            self.eat(T::Domain);
            let name = self.expect_name("a type parameter name");
            if self.at(T::Lt) {
                let start = self.start();
                self.bump();
                while !self.at_type_args_end() && !self.at_line_end() {
                    self.bump();
                }
                self.close_angle("`>` to close the type parameters");
                let span = self.span_from(start);
                self.not_in_language(
                    span,
                    &format!(
                        "type parameter `{}` can't take type parameters: there are no higher-kinded types (R08.5.3)",
                        name.text
                    ),
                    "pass the concrete type, or use a trait with an associated type",
                );
            }
            if self.eat(T::Colon) {
                let traits = self.bound_traits();
                if !value {
                    bounds.push(Bound {
                        name: name.clone(),
                        traits,
                    });
                }
            }
            let default = self.eat(T::Eq).then(|| self.ty());
            defaults.push(default);
            names.push(name);
            if !self.eat(T::Comma) {
                break;
            }
        }
        self.close_angle("`>` to close the type parameters");
        (names, defaults, bounds)
    }

    pub(super) fn generic_params(&mut self) -> (Vec<Name>, Vec<Bound>) {
        let (names, _, bounds) = self.generic_params_with_defaults();
        (names, bounds)
    }

    /// `Ord + Bounded`.
    fn bound_traits(&mut self) -> Vec<Type> {
        let mut traits = vec![self.ty_atom()];
        while self.eat(T::Plus) {
            traits.push(self.ty_atom());
        }
        traits
    }

    /// The built-in type a foreign type name here stands for. Rust's
    /// `Vec<T>` counts only with `<`: `std.geometry`'s `Vec` is a point.
    fn foreign_type_here(&self) -> Option<&'static str> {
        match self.text() {
            "Vec" | "ArrayList" | "Vector" if self.peek_n(1) == T::Lt => Some("List"),
            name => foreign_type(name),
        }
    }

    /// `List[Int]`, Python's generic brackets: read as `List<Int>`, with
    /// one error that turns the brackets into `<>`.
    fn bracket_type_args(&mut self, name: &str) -> Vec<Type> {
        if !self.at(T::LBracket) || !matches!(name, "List" | "Map" | "Set" | "HashMap" | "HashSet")
        {
            return Vec::new();
        }
        let open = self.span();
        self.bump();
        let mut args = Vec::new();
        while !self.at(T::RBracket) && !self.at_line_end() {
            args.push(self.ty());
            if !self.eat(T::Comma) {
                break;
            }
        }
        let close = self.span();
        if !self.eat(T::RBracket) {
            return args;
        }
        let mut diag = Diagnostic::error(
            codes::EXPECTED_TYPE,
            open,
            format!("CP Lang writes type arguments in `<>`: `{name}<...>`, not `{name}[...]`"),
        )
        .with_help("`[...]` is a list or an index; a type's arguments go in `<>`");
        diag.fixes.push(cpl_diag::Fix {
            title: "write `<...>`".into(),
            edits: vec![(open, "<".into()), (close, ">".into())],
        });
        self.error(diag);
        args
    }

    /// Parses `where T: Ord, U: Add + Mul` if it is here.
    pub(super) fn where_clause(&mut self, bounds: &mut Vec<Bound>) {
        if !self.at_ident("where") {
            return;
        }
        self.bump();
        loop {
            let name = self.expect_name("a type parameter name after `where`");
            self.expect(T::Colon, "`:` and the traits the type must have");
            let traits = self.bound_traits();
            bounds.push(Bound { name, traits });
            if !self.eat(T::Comma) {
                break;
            }
        }
    }
}

/// The built-in type another language's lowercase name stands for.
fn foreign_type(name: &str) -> Option<&'static str> {
    Some(match name {
        "int" | "long" | "ll" | "i64" | "i32" | "int64" | "integer" | "usize" => "Int",
        "string" | "str" => "Str",
        "bool" | "boolean" => "Bool",
        "float" | "double" | "f64" => "F64",
        "char" => "Char",
        "vector" | "vec" | "list" => "List",
        "map" | "dict" | "unordered_map" => "Map",
        "set" | "unordered_set" => "Set",
        _ => return None,
    })
}

//! Statements, blocks, and declarations.

use super::types::pattern_types;
use super::*;
use cpl_syntax::ast::{
    self, AssignOp, Block, Expr, ExprKind, FnBody, Pattern, PatternKind, Stmt, StmtKind,
};

impl<'a> Checker<'a> {
    /// `let g: [0..2, 0..3]Int = [[1, 2, 3], [4, 5, 6]]`: a list of rows
    /// nested as deep as the array has indices gives its cells.
    fn rows_fit(&mut self, declared: &Ty, row: &Ty) -> bool {
        let Ty::Array { index, elem } = declared else {
            return false;
        };
        let mut cell = row;
        for _ in 1..index.len() {
            match cell {
                Ty::Named { name, args } if name == "List" && args.len() == 1 => cell = &args[0],
                _ => return false,
            }
        }
        index.len() > 1 && self.fits(cell, elem)
    }

    fn array_fills(&mut self, declared: &Ty, init_ty: &Ty, init: &Expr) -> bool {
        match (declared, init_ty) {
            (Ty::Array { index, elem }, Ty::Named { name, args })
                if name == "List" && args.len() == 1 =>
            {
                if index.len() == 1 {
                    if let (ExprKind::List(items), Ty::Named { name: ename, .. }) =
                        (&init.kind, &index[0])
                    {
                        if let Some(Def::Enum { variants, .. }) =
                            self.type_def(ename).as_deref().cloned()
                        {
                            if variants.iter().all(|(_, fs)| fs.is_empty())
                                && items.len() != variants.len()
                            {
                                return false;
                            }
                        }
                    }
                }
                self.fits(&args[0], elem)
                    || self.fits(init_ty, elem)
                    || self.rows_fit(declared, &args[0])
            }
            (Ty::Array { elem, .. }, _) => {
                let mut cell = &**elem;
                let scalar = !matches!(init_ty, Ty::Array { .. })
                    && !matches!(init_ty, Ty::Named { name, .. } if name == "List");
                loop {
                    if self.fits(init_ty, cell) {
                        return true;
                    }
                    match (scalar, cell) {
                        (true, Ty::Array { elem, .. }) => cell = elem,
                        _ => return false,
                    }
                }
            }
            _ => false,
        }
    }

    /// Checks the statements of one top-level item.
    pub(crate) fn item(&mut self, stmts: &[Stmt]) {
        for replay in self.env.lent_before() {
            self.replay(&replay, stmts);
        }
        self.mutators = narrow::Mutators::of(stmts);
        let mut steps = cpl_syntax::order::Steps::default();
        steps.read(stmts, 0, &mut |f, x| self.call_writes(f, x));
        self.steps = steps;
        self.stmts(stmts, true);
    }

    /// Checks a block in its own scope. Its type is that of its last
    /// expression statement.
    pub(crate) fn block(&mut self, block: &Block) -> Ty {
        self.push_scope();
        let mark = self.lend_mark();
        let last = self.stmts(&block.stmts, false);
        if self.scopes.len() >= 2 {
            let outer_depth = self.scopes.len() - 2;
            if let Some(Stmt {
                kind: StmtKind::Expr(expr),
                ..
            }) = block.stmts.last()
            {
                self.check_expr_domain_escape(
                    expr,
                    &last,
                    outer_depth,
                    "returned from its declaring block",
                    &[],
                    false,
                );
            }
        }
        self.lend_reset(mark);
        self.pop_scope();
        last
    }

    /// Checks statements that run one after another. A slice or row
    /// bound by `let` keeps its array lent until the last statement that
    /// names it.
    fn stmts(&mut self, stmts: &[Stmt], top_level: bool) -> Ty {
        let mut last = Ty::Unit;
        let present = self.present.len();
        if !top_level {
            self.declare_local_types(stmts);
        }
        for (k, stmt) in stmts.iter().enumerate() {
            last = self.stmt(stmt, top_level);
            // `if x == none: return` leaves `x` holding a value.
            if !top_level {
                let held = narrow::after(stmt, &stmts[k + 1..], &mut |f, x| self.call_writes(f, x));
                self.present.extend(held.into_iter().map(String::from));
            }
            if let StmtKind::Let {
                mutable: false,
                pat:
                    Pattern {
                        kind: PatternKind::Name(name),
                        ..
                    },
                init: Some(init),
                ..
            } = &stmt.kind
            {
                self.begin_hold(stmt, name, init, &stmts[k + 1..]);
            }
            self.end_holds(stmt);
        }
        self.present.truncate(present);
        last
    }

    /// Binds the names a refutable `let ... else` pattern takes apart.
    fn bind_refutable(&mut self, pat: &Pattern, ty: &Ty, mutable: bool, keyword: &'static str) {
        let PatternKind::Tuple(items) = &pat.kind else {
            return self.bind(pat, ty, mutable, keyword);
        };
        for (i, item) in items.iter().enumerate() {
            let item_ty = match ty {
                Ty::Tuple(tys) if tys.len() == items.len() => tys[i].clone(),
                Ty::Record(fields) if fields.len() == items.len() => fields[i].1.clone(),
                Ty::Error => Ty::Error,
                _ => Ty::Unknown,
            };
            self.bind_refutable(item, &item_ty, mutable, keyword);
        }
    }

    /// Binds the names in `pat` to parts of a value of type `ty`.
    pub(crate) fn bind(&mut self, pat: &Pattern, ty: &Ty, mutable: bool, keyword: &'static str) {
        match &pat.kind {
            PatternKind::Name(_) | PatternKind::Domain(_) | PatternKind::Tuple(_) => {
                let effective_ty;
                let bind_ty = if matches!(pat.kind, PatternKind::Tuple(_)) {
                    if let Ty::Pos(dom) = ty {
                        if let Some(factors) = self.product_factors(dom) {
                            effective_ty =
                                Ty::Tuple(factors.into_iter().map(|(_, t)| t).collect());
                            &effective_ty
                        } else {
                            ty
                        }
                    } else {
                        ty
                    }
                } else {
                    ty
                };
                let mut names = Vec::new();
                pattern_types(pat, bind_ty, &mut |name, ty| names.push((name.to_string(), ty)));
                let mut spans = Vec::new();
                name_spans(pat, &mut spans);
                for ((name, ty), span) in names.into_iter().zip(spans) {
                    let kw = if matches!(ty, Ty::Domain(_)) {
                        "domain"
                    } else {
                        keyword
                    };
                    self.declare(&name, span, ty, mutable, kw);
                }
            }
            PatternKind::Variant { path, args } => {
                if path.len() == 1 && path[0].text == "some" {
                    let inner = match ty {
                        Ty::Optional(inner) => (**inner).clone(),
                        _ => Ty::Unknown,
                    };
                    if let Some(arg) = args.first() {
                        self.bind(arg, &inner, mutable, keyword);
                    }
                    return;
                }
                let fields = match path
                    .last()
                    .and_then(|name| self.global(&name.text))
                    .as_deref()
                {
                    Some(Def::Variant { fields, .. }) => fields.clone(),
                    _ => Vec::new(),
                };
                for (i, arg) in args.iter().enumerate() {
                    let ty = fields.get(i).cloned().unwrap_or(Ty::Unknown);
                    self.bind(arg, &ty, mutable, keyword);
                }
            }
            PatternKind::Literal(expr) => {
                self.infer(expr);
            }
            PatternKind::Or(alts) => {
                if let Some(first) = alts.first() {
                    self.bind(first, ty, mutable, keyword);
                }
            }
            PatternKind::Wildcard | PatternKind::Error => {}
        }
    }

    /// `let g: [Row, Col]Tile = read()` reads each `Tile` as text, so the
    /// enum says how its variants are written: nothing is read as a
    /// number or by guessing.
    fn readable(&mut self, declared: &Ty, init: &ast::Expr) {
        let reads = matches!(
            &init.kind,
            ExprKind::Call { callee, args }
                if args.is_empty() && matches!(&callee.kind, ExprKind::Name(name) if name == "read")
        );
        if !reads {
            return;
        }
        let mut todo = vec![declared];
        while let Some(ty) = todo.pop() {
            match ty {
                Ty::Array { elem, .. } | Ty::Optional(elem) => todo.push(elem),
                Ty::Tuple(items) => todo.extend(items),
                Ty::Named { name, args } => {
                    todo.extend(args);
                    let Some(Def::Enum {
                        variants,
                        tagged: false,
                        ..
                    }) = self.type_def(name).as_deref().cloned()
                    else {
                        continue;
                    };
                    if variants.iter().any(|(_, fields)| !fields.is_empty()) {
                        continue;
                    }
                    let tags: Vec<String> =
                        variants.iter().map(|(v, _)| format!("{v:?}")).collect();
                    let message = format!(
                        "reading a `{name}` needs to know how its variants are written in the input"
                    );
                    if let Some(diag) = self.error(codes::TYPE_MISMATCH, init.span, message) {
                        diag.help =
                            Some(format!("put `@tags({})` above the enum", tags.join(", ")));
                    }
                }
                _ => {}
            }
        }
    }

    /// What `let pat = init else: ...` binds: the value in each optional.
    /// A tuple of values is taken apart, `let (x, y) = (a, b) else: ...`.
    /// A refutable pattern, `let (gap, Some(best)) = e else: ...`, is matched
    /// against `e` itself.
    fn let_else(&mut self, pat: &ast::Pattern, init: &ast::Expr, ty: Ty) -> Ty {
        if refutable(pat) {
            return ty;
        }
        let inside = |ty: Ty| match ty {
            Ty::Optional(inner) => Ok(*inner),
            Ty::Error => Ok(Ty::Error),
            ty if ty.is_vague() || matches!(ty, Ty::Param(_)) => Ok(Ty::Unknown),
            ty => Err(ty),
        };
        match (&pat.kind, &init.kind, ty) {
            (ast::PatternKind::Tuple(_), ExprKind::Tuple(items), Ty::Tuple(tys))
                if items.len() == tys.len() =>
            {
                let mut out = Vec::new();
                for (item, ty) in items.iter().zip(tys) {
                    out.push(inside(ty).unwrap_or_else(|ty| {
                        let message = format!(
                            "let-else needs a ?T for `{}`, and it is {}",
                            self.quote(item.span),
                            ty.with_article()
                        );
                        self.error(codes::TYPE_MISMATCH, item.span, message);
                        Ty::Error
                    }));
                }
                Ty::Tuple(out)
            }
            (ast::PatternKind::Name(_) | ast::PatternKind::Tuple(_), _, ty) => inside(ty)
                .unwrap_or_else(|ty| {
                    let message = format!(
                        "let-else needs a ?T, and `{}` is {}",
                        self.quote(init.span),
                        ty.with_article()
                    );
                    self.error(codes::TYPE_MISMATCH, init.span, message);
                    Ty::Error
                }),
            (_, _, ty) => inside(ty).unwrap_or(Ty::Unknown),
        }
    }

    /// True for `g[r, ..]`: an index with a range among several.
    fn part_of_grid(e: &ast::Expr) -> bool {
        match &e.kind {
            ExprKind::Index { indices, .. } => {
                indices.len() > 1 && indices.iter().any(|i| matches!(i.kind, ExprKind::Range { .. }))
            }
            _ => false,
        }
    }

    /// `a[l..r] = b[l2..r2] + x`, or `x + b[..]`: a part takes another
    /// part worked with one value, cell for cell. Gives the type of the
    /// part on the right, a row; `None` when the statement is not one.
    fn part_map(&mut self, target: &ast::Expr, op: AssignOp, value: &ast::Expr) -> Option<Ty> {
        let is_part = |e: &ast::Expr| {
            matches!(&e.kind, ExprKind::Index { indices, .. }
                if indices.iter().filter(|i| matches!(i.kind, ExprKind::Range { step: None, .. })).count() == 1)
        };
        let ExprKind::Binary { op: bin, lhs, rhs, .. } = &value.kind else {
            return None;
        };
        let worked = matches!(
            bin,
            ast::BinOp::Add | ast::BinOp::Sub | ast::BinOp::Mul | ast::BinOp::BitAnd | ast::BinOp::BitOr | ast::BinOp::BitXor
        );
        if op != AssignOp::Set || !worked || !is_part(target) || is_part(lhs) == is_part(rhs) {
            return None;
        }
        let (part, one) = if is_part(lhs) { (lhs, rhs) } else { (rhs, lhs) };
        let (part_ty, one_ty) = (self.infer(part), self.infer(one));
        let cell = self.element_ty(&part_ty);
        let row = |ty: &Ty| matches!(ty, Ty::Array { .. }) || matches!(ty, Ty::Named { name, .. } if name == "List");
        if part_ty == Ty::Error || one_ty == Ty::Error {
            return Some(Ty::Error);
        }
        let number = cell.is_int() || cell == Ty::Float;
        if !row(&part_ty) || row(&one_ty) || !number || !(one_ty.is_vague() || self.fits(&one_ty, &cell)) {
            let message = format!(
                "`{}` works each cell of `{}` with one number, but `{}` is {}",
                self.quote(value.span),
                self.quote(part.span),
                self.quote(one.span),
                one_ty.with_article()
            );
            if let Some(diag) = self.error(codes::BAD_OPERANDS, value.span, message) {
                diag.help = Some("the cells and the value are one kind of number: `a[l..r] = b[l2..r2] + x`".to_string());
            }
            return Some(Ty::Error);
        }
        Some(part_ty)
    }

    /// `a max= b` on a row (an array of one dimension, or a list) goes
    /// cell by cell, so `b` is a row of the same type.
    pub(crate) fn row_keep(&mut self, target: &ast::Expr, target_ty: &Ty, value: &ast::Expr, value_ty: &Ty) {
        // `a[l..r] max= x`, `g[r, ..] min= x`: one value against every cell
        // of the part.
        let part = matches!(&target.kind, ExprKind::Index { indices, .. } if indices.iter().any(|i| matches!(i.kind, ExprKind::Range { .. })));
        let cell = match target_ty {
            Ty::Array { elem, .. } => Some(&**elem),
            Ty::Named { name, args } if name == "List" && args.len() == 1 => Some(&args[0]),
            _ => None,
        };
        if let (true, Some(cell)) = (part, cell) {
            let row = matches!(value_ty, Ty::Array { .. }) || matches!(value_ty, Ty::Named { name, .. } if name == "List");
            if !row && !value_ty.is_vague() && self.fits(value_ty, cell) {
                return;
            }
        }
        // `g[r, ..] max= g[v, ..]`: a row of a grid against cells of its kind.
        if let (true, Ty::Array { elem: want, .. }) = (Self::part_of_grid(target), target_ty) {
            let got = match value_ty {
                Ty::Array { index, elem } if index.len() == 1 || Self::part_of_grid(value) => {
                    Some((**elem).clone())
                }
                Ty::Named { name, args } if name == "List" && args.len() == 1 => {
                    Some(args[0].clone())
                }
                other if other.is_vague() || *other == Ty::Error => return,
                _ => None,
            };
            if got
                .as_ref()
                .is_some_and(|got| got.is_vague() || self.fits(got, want))
            {
                return;
            }
            let message = format!(
                "`{}` is a row of {want}, so the other side is one too, cell for cell; `{}` is {}",
                self.quote(target.span),
                self.quote(value.span),
                match got {
                    Some(got) => format!("a row of {got}"),
                    None => value_ty.with_article(),
                }
            );
            self.error(codes::TYPE_MISMATCH, value.span, message);
            return;
        }
        let row = match target_ty {
            Ty::Array { index, .. } => index.len() == 1,
            Ty::Named { name, args } => name == "List" && args.len() == 1,
            _ => false,
        };
        if !row {
            return;
        }
        self.expect_fits(value_ty, target_ty, value, |c, got, want| {
            format!(
                "`{}` is {}, so the other side is one too, cell for cell; `{}` is {}",
                c.quote(target.span),
                want.with_article(),
                c.quote(value.span),
                got.with_article()
            )
        });
    }

    /// `m[k] f= x` on a map folds `x` into the entry, and a key that is
    /// not there starts from the identity of `f` (decisions §13). `min`
    /// and `max` store `x`; an operation with no identity is an error.
    fn map_fold(&mut self, target: &ast::Expr, op: AssignOp, value: &ast::Expr) {
        let ExprKind::Index { base, indices } = &target.kind else {
            return;
        };
        let fold = match (op, &value.kind) {
            (AssignOp::Binary(op), _) => {
                if matches!(
                    op,
                    ast::BinOp::Add
                        | ast::BinOp::Sub
                        | ast::BinOp::Mul
                        | ast::BinOp::BitOr
                        | ast::BinOp::BitXor
                        // `m[k] ??= init` is the entry itself: a missing
                        // key gets `init`.
                        | ast::BinOp::Coalesce
                ) {
                    return;
                }
                op.text().to_string()
            }
            // `m[k] f= x`, which the parser wrote as `m[k] = f(m[k], x)`.
            (AssignOp::Set, ExprKind::Call { callee, args })
                if args.first().is_some_and(|first| first.value == *target) =>
            {
                match &callee.kind {
                    ExprKind::Name(name) if !matches!(name.as_str(), "gcd" | "lcm") => name.clone(),
                    _ => return,
                }
            }
            _ => return,
        };
        let map = self.quietly(|c| c.infer(base));
        let Ty::Named { name, args } = &map else {
            return;
        };
        let plain = matches!(args.as_slice(), [_, value] if !matches!(value, Ty::Optional(_)) && !value.is_vague());
        if !matches!(name.as_str(), "Map" | "HashMap") || !plain {
            return;
        }
        let (target_text, key) = (self.quote(target.span), self.quote(indices[0].span));
        let message = format!(
            "`{target_text} {fold}= x` may meet a key that is not in the map yet, and `{fold}` has no identity to start it from"
        );
        let base = self.quote(base.span);
        let help = match op {
            AssignOp::Binary(_) => format!(
                "say what a missing key means: `{target_text} = ({base}.get({key}) ?? start) {fold} x`"
            ),
            _ => format!(
                "say what a missing key means: `{target_text} = {fold}({base}.get({key}) ?? start, x)`"
            ),
        };
        if let Some(diag) = self.error(codes::BAD_OPERANDS, target.span, message) {
            diag.help = Some(help);
        }
    }

    /// A domain written with two literal ends holds at most 2^31 - 1
    /// values: a position is an `i32` in the generated code.
    fn domain_size(&mut self, def: &ast::Expr) {
        let ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
            ..
        } = &def.kind
        else {
            return;
        };
        let (ExprKind::Int(lo), ExprKind::Int(hi)) = (&lo.kind, &hi.kind) else {
            return;
        };
        let len = *hi - *lo + i128::from(*inclusive);
        if len > i128::from(i32::MAX) {
            let message = format!("a domain holds at most 2^31 - 1 values, and this one has {len}");
            self.error(codes::TYPE_MISMATCH, def.span, message);
        }
    }

    fn stmt(&mut self, stmt: &Stmt, top_level: bool) -> Ty {
        // A statement is straight-line code again, whatever it is inside;
        // an `if` or `match` standing as a statement is one too.
        let branch = match &stmt.kind {
            StmtKind::Expr(expr) => {
                if matches!(expr.kind, ExprKind::If { .. } | ExprKind::Match { .. }) {
                    self.plain_if = Some(expr.span);
                }
                self.branch
            }
            _ => std::mem::take(&mut self.branch),
        };
        let ty = self.stmt_kind(stmt, top_level);
        self.branch = branch;
        ty
    }

    fn stmt_kind(&mut self, stmt: &Stmt, top_level: bool) -> Ty {
        match &stmt.kind {
            StmtKind::Let {
                mutable,
                pat,
                ty,
                init,
                otherwise,
                guard,
            } => {
                let keyword = if *mutable { "var" } else { "let" };
                let declared = ty.as_ref().map(|ty| self.resolve_type(ty));
                let prev_bounds_range = self.expected_bounds_range;
                if declared.as_ref().is_some_and(Self::expects_bounds_range) {
                    self.expected_bounds_range = true;
                }
                let prev_piece = std::mem::replace(
                    &mut self.expected_piece,
                    declared.as_ref().is_some_and(Self::expects_piece),
                );
                if let (Some(declared), Some(Expr { kind: ExprKind::Call { callee, args }, .. })) = (&declared, init) {
                    let reads = match &callee.kind {
                        ExprKind::Name(n) if n == "read" => args.is_empty(),
                        ExprKind::Name(n) if n == "ask" => args.iter().all(|a| self.fixed_size(&a.value)),
                        _ => false,
                    };
                    if reads && declared.is_fixed_size() {
                        self.fixed_io.push(callee.span);
                    }
                }
                // §127: `let f = (n: Int) -> Int => ...` may refer to `f` recursively.
                // `var f = ...` cannot refer to itself recursively.
                let mut rec_scope = false;
                if let (
                    PatternKind::Name(vname),
                    Some(Expr {
                        kind:
                            ExprKind::Lambda {
                                params,
                                param_tys,
                                ret_ty,
                                body,
                                ..
                            },
                        ..
                    }),
                ) = (&pat.kind, init)
                {
                    if *mutable {
                        struct Mentions<'a> {
                            target: &'a str,
                            found: Option<Span>,
                        }
                        impl cpl_syntax::visit::Visitor for Mentions<'_> {
                            fn expr(&mut self, expr: &Expr) {
                                if self.found.is_none() {
                                    if let ExprKind::Name(n) = &expr.kind {
                                        if n == self.target {
                                            self.found = Some(expr.span);
                                        }
                                    }
                                }
                            }
                        }
                        let mut m = Mentions {
                            target: vname,
                            found: None,
                        };
                        cpl_syntax::visit::walk_expr(body, &mut m);
                        if let Some(at) = m.found {
                            let keyword = self.src[..pat.span.start as usize]
                                .trim_end()
                                .strip_suffix("var")
                                .map(|before| Span::new(before.len() as u32, before.len() as u32 + 3));
                            if let Some(diag) = self.error(
                                codes::VAR_LAMBDA_RECURSION,
                                at,
                                format!(
                                    "`var {vname}` cannot refer to itself recursively; bind it with `let`"
                                ),
                            ) {
                                if let Some(keyword) = keyword {
                                    diag.fixes.push(cpl_diag::Fix {
                                        title: format!("declare `{vname}` with `let`"),
                                        edits: vec![(keyword, "let".into())],
                                    });
                                }
                            }
                        }
                    } else {
                        let rec_ty = if let Some(decl @ Ty::Fn { .. }) = &declared {
                            decl.clone()
                        } else {
                            let sig_params = params
                                .iter()
                                .enumerate()
                                .map(|(i, _)| {
                                    let pty = param_tys
                                        .get(i)
                                        .and_then(|t| t.as_ref())
                                        .map(|t| self.resolve_type(t))
                                        .unwrap_or(Ty::Unknown);
                                    crate::ty::Param {
                                        name: String::new(),
                                        ty: pty,
                                        mutable: false,
                                        has_default: false,
                                        named_only: false,
                                        variadic: false,
                                        lazy: false,
                                    }
                                })
                                .collect();
                            let rty = ret_ty
                                .as_ref()
                                .map(|t| self.resolve_type(t))
                                .unwrap_or(Ty::Unknown);
                            Ty::Fn {
                                params: sig_params,
                                ret: Box::new(rty),
                            }
                        };
                        self.push_scope();
                        rec_scope = true;
                        self.bind(pat, &rec_ty, false, "let");
                    }
                }
                let mut init_ty = init.as_ref().map(|init| self.infer(init));
                self.expected_bounds_range = prev_bounds_range;
                self.expected_piece = prev_piece;
                if rec_scope {
                    self.pop_scope();
                }
                if let (Some(declared), Some(init)) = (&declared, init) {
                    self.readable(declared, init);
                }
                // `let v = u else: return`: past here `v` is the value in `u`.
                if let Some(otherwise) = otherwise {
                    init_ty = match (init_ty, init) {
                        (Some(ty), Some(init)) => Some(self.let_else(pat, init, ty)),
                        (ty, _) => ty,
                    };
                    self.block(otherwise);
                }
                // `var seen: [V]Bool = false` fills the array with the value.
                // A comprehension builds the array when one is declared.
                let fills = match (&declared, &init_ty, init) {
                    (Some(decl), Some(ity), Some(init_expr)) => self.array_fills(decl, ity, init_expr),
                    _ => false,
                };
                if let (Some(declared), Some(init_ty), Some(init), false) =
                    (&declared, &init_ty, init, fills)
                {
                    let name = self.quote(pat.span);
                    self.expect_fits(init_ty, declared, init, |c, got, want| {
                        format!(
                            "`{name}` is declared {}, but `{}` is {}",
                            want.with_article(),
                            c.quote(init.span),
                            got.with_article()
                        )
                    });
                }
                let guessed = declared.is_none()
                    && self.sink.encloses_poison(stmt.span)
                    && crate::db::head_leaves_open(self.src, stmt);
                let ty = if guessed {
                    Ty::Error
                } else {
                    declared.or(init_ty.clone()).unwrap_or(Ty::Error)
                };
                // §128: `let b = a` / `var b = a` on a heap variable moves `a`.
                if otherwise.is_none() && !fills {
                    if let Some(Expr {
                        kind: ExprKind::Name(src),
                        span: src_span,
                    }) = init
                    {
                        let is_owned = self.is_owned_movable_var(src);
                        let src_is_heap = init_ty
                            .as_ref()
                            .is_some_and(|t| self.is_heap_type(t));
                        if is_owned && src_is_heap {
                            self.mark_moved(src, *src_span);
                        }
                    }
                }
                if let (true, Some(init)) = (*mutable, init) {
                    let what = format!("`var {}`", self.quote(pat.span));
                    self.no_escape(init, &what);
                }
                // A new binding of a name: what was known about the old
                // one no longer applies.
                let mut bound = Vec::new();
                pattern_types(pat, &ty, &mut |name, _| bound.push(name.to_string()));
                self.drop_facts(&bound);
                let is_mut = *mutable
                    || matches!(
                        init,
                        Some(Expr {
                            kind: ExprKind::Mut(_),
                            ..
                        })
                    );
                match super::types::domain_binding(pat) {
                    // `let LIM = 100`: with no `else` there is no variant
                    // to match, so a capitalized name is bound to the value.
                    Some(name)
                        if otherwise.is_none()
                            && !matches!(
                                self.global(name).as_deref(),
                                Some(Def::Variant { .. })
                            ) =>
                    {
                        self.declare(name, pat.span, ty.clone(), is_mut, keyword);
                    }
                    _ if otherwise.is_some() && refutable(pat) => {
                        self.bind_refutable(pat, &ty, is_mut, keyword)
                    }
                    _ => self.bind(pat, &ty, is_mut, keyword),
                }
                if let Some(guard) = guard {
                    let ty = self.infer(guard);
                    self.require_bool(&ty, guard, "`where`");
                }
            }
            StmtKind::Const {
                name,
                generics,
                bounds,
                ty,
                init,
            } => {
                let outer = self.generics.len();
                self.generics
                    .extend(generics.iter().map(|g| g.text.clone()));
                self.bounds_exist(bounds);
                let declared = ty.as_ref().map(|ty| self.resolve_type(ty));
                let init_ty = init.as_ref().map(|init| self.infer(init));
                if let (Some(declared), Some(init_ty), Some(init)) = (&declared, &init_ty, init) {
                    self.expect_fits(init_ty, declared, init, |c, got, want| {
                        format!(
                            "`{}` is declared {}, but `{}` is {}",
                            name.text,
                            want.with_article(),
                            c.quote(init.span),
                            got.with_article()
                        )
                    });
                }
                let ty = declared.or(init_ty).unwrap_or(Ty::Error);
                self.generics.truncate(outer);
                self.declare(&name.text, name.span, ty, false, "const");
            }
            StmtKind::Fn(f) => {
                // `@memo(into: dp)` on a fn inside a fn or a loop: the
                // table is a `var` where the fn stands.
                if let (false, Some((table, at))) = (top_level, crate::check::memo_table(stmt)) {
                    if let Ty::Fn { params, ret } = self.fn_sig(f, None) {
                        let ty = Ty::Array {
                            index: params.iter().map(|p| p.ty.clone()).collect(),
                            elem: Box::new(Ty::Optional(Box::new(ret.widen()))),
                        };
                        self.declare(table, at, ty, true, "var");
                    }
                }
                self.fn_decl(f, None, !top_level, memo_of(stmt))
            }
            StmtKind::Struct(s) => {
                let generics: Vec<String> = s.generics.iter().map(|g| g.text.clone()).collect();
                let self_ty = Ty::Named {
                    name: s.name.text.clone(),
                    args: generics.iter().cloned().map(Ty::Param).collect(),
                };
                let body = self.enter_body(&generics, self_ty.clone());
                self.bounds_exist(&s.bounds);
                self.owned(&s.name.text, &s.methods);
                for field in &s.fields {
                    let ty = self.field_type(field);
                    self.hovers.push(HoverEntry {
                        span: field.name.span,
                        name: field.name.text.clone(),
                        ty: ty.clone(),
                        keyword: "field",
                        decl: Some(field.name.span),
                        owner: None,
                    });
                    if let Some(default) = &field.default {
                        let default_ty = self.infer(default);
                        if !self.array_fills(&ty, &default_ty, default) {
                            self.expect_fits(&default_ty, &ty, default, |c, got, want| {
                                format!(
                                    "`{}` is {}, but its default `{}` is {}",
                                    field.name.text,
                                    want.with_article(),
                                    c.quote(default.span),
                                    got.with_article()
                                )
                            });
                        }
                    }
                }
                self.type_hover(&s.name);
                self.methods(&s.methods, &self_ty);
                self.leave_body(body);
            }
            StmtKind::Enum(e) => {
                let generics: Vec<String> = e.generics.iter().map(|g| g.text.clone()).collect();
                let self_ty = Ty::Named {
                    name: e.name.text.clone(),
                    args: generics.iter().cloned().map(Ty::Param).collect(),
                };
                let body = self.enter_body(&generics, self_ty.clone());
                for variant in &e.variants {
                    let fields: Vec<Ty> = variant
                        .fields
                        .iter()
                        .map(|(_, ty)| self.resolve_type(ty))
                        .collect();
                    let ty = Def::Variant {
                        of: e.name.text.clone(),
                        fields,
                    }
                    .value_ty(&variant.name.text);
                    self.hovers.push(HoverEntry {
                        span: variant.name.span,
                        name: variant.name.text.clone(),
                        ty,
                        keyword: "global",
                        decl: Some(variant.name.span),
                        owner: None,
                    });
                }
                self.type_hover(&e.name);
                self.methods(&e.methods, &self_ty);
                self.leave_body(body);
            }
            StmtKind::Domain { name, def } => {
                self.infer(def);
                self.domain_size(def);
                self.type_hover(name);
                if !top_level {
                    let info = self.domain_info(def);
                    if !info.factors.is_empty() {
                        self.local_product_domains
                            .insert(name.text.clone(), info.factors);
                    }
                    self.declare(
                        &name.text,
                        name.span,
                        Ty::Domain(name.text.clone()),
                        false,
                        "domain",
                    );
                }
            }
            StmtKind::TypeAlias { name, generics, ty } => {
                let outer = self.generics.len();
                self.generics
                    .extend(generics.iter().map(|g| g.text.clone()));
                let aliased = self.resolve_type(ty);
                self.generics.truncate(outer);
                self.type_hover(name);
                if !top_level {
                    self.scopes.last_mut().unwrap().insert(
                        name.text.clone(),
                        Local {
                            ty: aliased,
                            mutable: false,
                            span: name.span,
                            keyword: "type",
                        },
                    );
                }
            }
            StmtKind::Trait {
                name,
                generics,
                supers,
                body,
            } => {
                let mut generics: Vec<String> = generics.iter().map(|g| g.text.clone()).collect();
                for stmt in &body.stmts {
                    if let StmtKind::TypeAlias { name, .. } = &stmt.kind {
                        generics.push(name.text.clone());
                    }
                }
                let self_ty = Ty::Param("Self".into());
                let scope = self.enter_body(&generics, self_ty.clone());
                for sup in supers {
                    self.trait_exists(sup);
                }
                self.type_hover(name);
                self.owned(&name.text, &body.stmts);
                self.methods(&body.stmts, &self_ty);
                self.leave_body(scope);
            }
            StmtKind::Impl {
                trait_name,
                target,
                bounds,
                body,
            } => {
                let mut generics = self.impl_generics(target, bounds);
                for stmt in &body.stmts {
                    if let StmtKind::TypeAlias { name, .. } = &stmt.kind {
                        generics.push(name.text.clone());
                    }
                }
                let scope = self.enter_body(&generics, Ty::Unknown);
                let self_ty = self.resolve_type(target);
                self.self_ty = Some(self_ty.clone());
                self.bounds_exist(bounds);
                if let Some(trait_name) = trait_name {
                    self.impl_matches_trait(trait_name, target, &body.stmts);
                    if type_head(trait_name).as_deref() == Some("Index") {
                        self.index_size(&body.stmts);
                    }
                }
                let owner = type_head(target).unwrap_or_default();
                let owned_from_struct: Vec<(String, Ty)> =
                    if let Some(Def::Struct { owned, .. }) = self.type_def(&owner).as_deref() {
                        owned.clone()
                    } else {
                        Vec::new()
                    };
                for (oname, oty) in owned_from_struct {
                    self.scopes.last_mut().unwrap().insert(
                        oname,
                        Local {
                            ty: oty,
                            mutable: false,
                            span: target.span,
                            keyword: "let",
                        },
                    );
                }
                self.owned(&owner, &body.stmts);
                self.methods(&body.stmts, &self_ty);
                self.leave_body(scope);
            }
            StmtKind::Use { .. } | StmtKind::Break(_) | StmtKind::Continue(_) | StmtKind::Error => {
            }
            StmtKind::Expr(expr) => return self.infer(expr),
            StmtKind::Assign { target, op, value } => {
                let prev_assigning = self.assigning_name;
                if *op == AssignOp::Set && matches!(target.kind, ExprKind::Name(_)) {
                    self.assigning_name = Some(target.span);
                }
                let target_ty = self.infer(target);
                let prev_bounds_range = self.expected_bounds_range;
                if Self::expects_bounds_range(&target_ty) {
                    self.expected_bounds_range = true;
                }
                self.assigning_name = prev_assigning;
                let value_ty = match self.part_map(target, *op, value) {
                    Some(ty) => ty,
                    None => self.infer(value),
                };
                self.expected_bounds_range = prev_bounds_range;
                if *op == AssignOp::Set {
                    if let ExprKind::Name(tname) = &target.kind {
                        self.moved.remove(tname);
                        if let ExprKind::Name(sname) = &value.kind {
                            let is_owned = self.is_owned_movable_var(sname);
                            if tname != sname && is_owned && self.is_heap_type(&value_ty) {
                                self.mark_moved(sname, value.span);
                            }
                        }
                    }
                }
                self.optional_fold(target, &target_ty, *op);
                self.map_fold(target, *op, value);
                self.assign_alias(target, *op, value);
                self.require_mutable(target, "assigned");
                self.check_access(target, true);
                if let Some((root, _)) = Self::root_name(target) {
                    let target_depth = self.var_depth(root);
                    let how = format!("assigned to `{root}` from an outer scope");
                    self.check_expr_domain_escape(value, &value_ty, target_depth, &how, &[], false);
                    self.drop_facts(&[root.to_string()]);
                }
                // `a[l..r] = 0`, `g[r, ..] = 0`: one value for every cell
                // of the part.
                let part = matches!(&target.kind, ExprKind::Index { indices, .. }
                    if indices.iter().any(|index| matches!(index.kind, ExprKind::Range { step: None, .. })));
                if *op == AssignOp::Set && !part {
                    self.no_escape(value, "this assignment");
                }
                if matches!(op, AssignOp::Max | AssignOp::Min) {
                    self.row_keep(target, &target_ty, value, &value_ty);
                }
                let row = |ty: &Ty| {
                    matches!(ty, Ty::Array { .. }) || matches!(ty, Ty::Named { name, .. } if name == "List")
                };
                let target_ty = if *op == AssignOp::Set && part && row(&target_ty) && !row(&value_ty) {
                    self.element_ty(&target_ty)
                } else {
                    target_ty
                };
                let target_ty =
                    if *op == AssignOp::Set && part && row(&target_ty) && !row(&value_ty) {
                        self.element_ty(&target_ty)
                    } else {
                        target_ty
                    };
                // `dp[k, ..] = row`, `a[l..r] = [x, y]`: a part of an array takes
                // a row, cell for cell.
                let cells = match (&target_ty, &value_ty) {
                    (
                        Ty::Array { index, .. },
                        Ty::Array {
                            index: theirs,
                            elem,
                        },
                    ) if (index.len() > 1 || index != theirs) && theirs.len() == 1 => {
                        Some((**elem).clone())
                    }
                    (Ty::Array { .. }, Ty::Named { name, args })
                        if name == "List" && args.len() == 1 =>
                    {
                        Some(args[0].clone())
                    }
                    _ => None,
                };
                if let (AssignOp::Set, true, Some(cells)) = (*op, part, cells) {
                    let want = self.element_ty(&target_ty);
                    if !cells.is_vague() && !self.fits(&cells, &want) {
                        let message = format!(
                            "`{}` is a row of {want}, but `{}` is {}",
                            self.quote(target.span),
                            self.quote(value.span),
                            value_ty.with_article()
                        );
                        self.error(codes::TYPE_MISMATCH, value.span, message);
                    }
                } else if *op == AssignOp::Set {
                    self.expect_fits(&value_ty, &target_ty, value, |c, got, want| {
                        format!(
                            "`{}` is {}, but `{}` is {}",
                            c.quote(target.span),
                            want.with_article(),
                            c.quote(value.span),
                            got.with_article()
                        )
                    });
                }
            }
            StmtKind::While {
                cond,
                body,
                else_body,
            } => {
                self.push_scope();
                self.drop_facts_written(body, None);
                // The test runs again each round, so a `?` in it is not
                // in straight-line code.
                self.branch += 1;
                self.cond(cond);
                self.branch -= 1;
                let mark = self.lend_mark();
                self.add_cond_facts(cond, false);
                self.block(body);
                self.lend_reset(mark);
                self.pop_scope();
                if let Some(else_body) = else_body {
                    self.branch += 1;
                    self.push_scope();
                    self.block(else_body);
                    self.pop_scope();
                    self.branch -= 1;
                }
            }
            StmtKind::For {
                pat,
                iter,
                body,
                else_body,
                ..
            } if self.reflected_values(iter).is_some() => {
                self.fields_loop(pat, iter, body);
                if let Some(else_body) = else_body {
                    self.branch += 1;
                    self.push_scope();
                    self.block(else_body);
                    self.pop_scope();
                    self.branch -= 1;
                }
            }
            StmtKind::For {
                pat,
                iter,
                body,
                else_body,
                ..
            } => {
                let prev_iterating = std::mem::replace(
                    &mut self.iterating_range,
                    matches!(iter.kind, ExprKind::Range { .. }),
                );
                let iter_ty = self.infer(iter);
                self.iterating_range = prev_iterating;
                let elem = self.element_ty(&iter_ty);
                self.push_scope();
                self.drop_facts_written(body, Some(pat));
                self.bind(pat, &elem, false, "for");
                let mark = self.lend_mark();
                self.hold_iterated(stmt, iter, &iter_ty);
                self.block(body);
                self.lend_reset(mark);
                self.pop_scope();
                if let Some(else_body) = else_body {
                    self.branch += 1;
                    self.push_scope();
                    self.block(else_body);
                    self.pop_scope();
                    self.branch -= 1;
                }
            }
            StmtKind::Return(value) => {
                let value_ty = value.as_ref().map(|v| self.infer(v));
                if let (Some(value), Some(ty), true) =
                    (value, value_ty.as_ref(), self.fns.is_empty())
                {
                    if !matches!(ty, Ty::Unit | Ty::Unknown | Ty::Error) {
                        // Decision 20.3: there is no caller to receive a non-unit value.
                        let diag = self.error(
                            codes::UNEXPECTED_TOKEN,
                            value.span,
                            "a top-level `return` takes no value".to_string(),
                        );
                        if let Some(diag) = diag {
                            diag.help = Some(
                                "write a bare `return`: it ends the program, or the test case below `@!multitest`"
                                    .to_string(),
                            );
                        }
                    }
                }
                if let (Some(value), Some(value_ty)) = (value, value_ty) {
                    if !(self.fns.is_empty() && matches!(value_ty, Ty::Unit)) {
                        self.returns(value, &value_ty);
                        self.no_escape(value, "`return`");
                    }
                }
            }
            StmtKind::Yield { value, .. } => {
                self.infer(value);
            }
            StmtKind::Assert { cond, message } => {
                let ty = self.infer(cond);
                self.require_bool(&ty, cond, "`assert`");
                self.add_facts(cond, false);
                if let Some(message) = message {
                    self.infer(message);
                }
            }
            StmtKind::Unchecked(body) => return self.block(body),
            StmtKind::Law { params, body, .. } => {
                self.push_scope();
                for param in params {
                    let ty = match &param.ty {
                        Some(ty) => self.resolve_type(ty),
                        None => Ty::Unknown,
                    };
                    self.declare(&param.name.text, param.name.span, ty, false, "param");
                }
                self.infer(body);
                self.pop_scope();
            }
            StmtKind::Lend { place, .. } => {
                self.infer(place);
                self.lend_in_place(stmt);
            }
        }
        Ty::Unit
    }

    /// Checks a returned value against the enclosing fn's declared type.
    fn returns(&mut self, value: &Expr, value_ty: &Ty) {
        let Some((name, mut ret, declared, fn_depth)) = self.fns.last().cloned() else {
            return;
        };
        // `print(..)` is checked loosely, as a call of unknown type: it
        // gives nothing.
        let prints = matches!(&value.kind, ExprKind::Call { callee, .. }
            if matches!(&callee.kind, ExprKind::Name(name) if name == "print"));
        if let (Some(gives), false) = (self.gives.last_mut(), *value_ty == Ty::Unit || prints) {
            *gives = true;
        }
        // Existential domains declared in the return type may escape.
        let mut allowed = ret.existential_domains();
        // A returned user-defined struct that carries a fresh domain as a type
        // argument (e.g. `Compressed<D, T>`, `Rooted<V, P>`) packs that domain
        // existentially as a value-owned domain (`xs.D`, `t.Pre`).
        if let Ty::Named { name: sname, args } = value_ty {
            if matches!(self.global(sname).as_deref(), Some(Def::Struct { .. })) {
                for arg in args {
                    if let Ty::Pos(d) | Ty::Domain(d) = arg {
                        allowed.push(d.clone());
                    }
                }
            }
        }
        // If `ret` has existential `domain P` slots and `value_ty` returns a local
        // domain `Q` in that slot, rename `P -> Q` for type checking and allow `Q`.
        if declared {
            let mut pairs: Vec<(String, String)> = Vec::new();
            match (&ret, value_ty) {
                (Ty::Domain(sig_d), Ty::Domain(val_d)) if sig_d != val_d => {
                    pairs.push((sig_d.clone(), val_d.clone()));
                }
                (Ty::Tuple(rt), Ty::Tuple(vt)) => {
                    for (r, v) in rt.iter().zip(vt.iter()) {
                        if let (Ty::Domain(sig_d), Ty::Domain(val_d)) = (r, v) {
                            if sig_d != val_d {
                                pairs.push((sig_d.clone(), val_d.clone()));
                            }
                        }
                    }
                }
                (Ty::Record(rf), Ty::Record(vf)) => {
                    for ((_, r), (_, v)) in rf.iter().zip(vf.iter()) {
                        if let (Ty::Domain(sig_d), Ty::Domain(val_d)) = (r, v) {
                            if sig_d != val_d {
                                pairs.push((sig_d.clone(), val_d.clone()));
                            }
                        }
                    }
                }
                (Ty::Record(rf), Ty::Tuple(vt)) => {
                    for ((_, r), v) in rf.iter().zip(vt.iter()) {
                        if let (Ty::Domain(sig_d), Ty::Domain(val_d)) = (r, v) {
                            if sig_d != val_d {
                                pairs.push((sig_d.clone(), val_d.clone()));
                            }
                        }
                    }
                }
                _ => {}
            }
            for (sig_d, val_d) in pairs {
                allowed.push(val_d.clone());
                ret = ret.rename_domain(&sig_d, &val_d);
            }
        }
        let how = if name.is_empty() {
            "returned from closure".to_string()
        } else {
            format!("returned from `{name}`")
        };
        self.check_expr_domain_escape(value, value_ty, fn_depth, &how, &allowed, !name.is_empty());
        if !declared {
            return;
        }
        self.expect_fits(value_ty, &ret, value, |c, got, want| {
            format!(
                "`{name}` returns {}, but `{}` is {}",
                want.with_article(),
                c.quote(value.span),
                got.with_article()
            )
        });
    }

    fn type_hover(&mut self, name: &ast::Name) {
        self.hovers.push(HoverEntry {
            span: name.span,
            name: name.text.clone(),
            ty: Ty::TypeName(name.text.clone()),
            keyword: "type",
            decl: Some(name.span),
            owner: None,
        });
    }

    /// Checks the fns, laws and constants of a struct, enum, trait or impl
    /// body. The struct's fields can be named in them without `self.`.
    fn methods(&mut self, methods: &[Stmt], self_ty: &Ty) {
        let fields = match self_ty {
            Ty::Named { name, .. } => match self.global(name).as_deref() {
                Some(Def::Struct { fields, .. }) => fields.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        for method in methods {
            match &method.kind {
                StmtKind::Fn(f) => self.fn_decl(f, Some(self_ty), false, memo_of(method)),
                StmtKind::Law { name, .. } => {
                    self.push_scope();
                    for field in &fields {
                        self.scopes.last_mut().unwrap().insert(
                            field.name.clone(),
                            Local {
                                ty: field.ty.clone(),
                                mutable: false,
                                span: name.span,
                                keyword: "self",
                            },
                        );
                    }
                    self.stmt(method, false);
                    self.pop_scope();
                }
                // Owned domains and constants were declared on entry; this
                // checks their values.
                StmtKind::Domain { def, .. } => {
                    self.infer(def);
                }
                StmtKind::Const {
                    init: Some(init), ..
                }
                | StmtKind::Let {
                    init: Some(init), ..
                } => {
                    self.infer(init);
                }
                _ => {
                    self.stmt(method, false);
                }
            }
        }
    }

    /// The `K` of an `impl Index`: how many values the type has. `const K`
    /// is known at compile time; `let K` is evaluated once, where the
    /// impl is declared, and frozen, so it can't read a `var`
    /// (plan/19 §3.1).
    fn index_size(&mut self, body: &[Stmt]) {
        for stmt in body {
            match &stmt.kind {
                StmtKind::Const {
                    name,
                    init: Some(init),
                    ..
                } if name.text == "K" => {
                    let known = matches!(self.bounds(init), (Some(lo), Some(hi)) if lo == hi);
                    if !known {
                        let value = self.quote(init.span);
                        let message = format!(
                            "`const K = {value}`: `const` is compile-time only, and `{value}` is known only at run time"
                        );
                        if let Some(diag) = self.error(codes::BAD_INDEX_IMPL, stmt.span, message) {
                            diag.help = Some(format!(
                                "write `let K = {value}`; an associated `let` is evaluated once, where the impl is declared, and is frozen from then on"
                            ));
                        }
                    }
                }
                StmtKind::Let {
                    pat,
                    init: Some(init),
                    ..
                } if super::types::domain_binding(pat) == Some("K") => {
                    for name in super::lends::names_in(init) {
                        let mutable = match self.local(&name) {
                            Some(local) => local.mutable,
                            None => matches!(
                                self.global(&name).as_deref(),
                                Some(Def::Value { mutable: true, .. })
                            ),
                        };
                        if mutable {
                            let message = format!(
                                "`K` can't be sized by the var `{name}`: it could change after arrays are made"
                            );
                            if let Some(diag) =
                                self.error(codes::BAD_INDEX_IMPL, init.span, message)
                            {
                                diag.help = Some(format!("freeze it first: `let n = {name}`"));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Every trait a bound names must exist: `where T: Ord + Bounded`.
    fn bounds_exist(&mut self, bounds: &[ast::Bound]) {
        for bound in bounds {
            for trait_ty in &bound.traits {
                self.trait_exists(trait_ty);
            }
        }
    }

    /// Reports a trait name that is not a trait. Returns the trait's
    /// methods when it is declared in cp source.
    fn trait_exists(&mut self, trait_ty: &ast::Type) -> Option<Vec<crate::def::TraitMethod>> {
        let ast::TypeKind::Named { name, args } = &trait_ty.kind else {
            return None;
        };
        for arg in args {
            self.resolve_type(arg);
        }
        match self.global(&name.text).as_deref() {
            Some(Def::Trait { methods, .. }) => {
                self.refs.push(GlobalRef {
                    span: name.span,
                    name: name.text.clone(),
                    top_level: false,
                });
                return Some(methods.clone());
            }
            Some(_) => {
                let message = format!("`{}` is not a trait", name.text);
                self.error(codes::UNKNOWN_TYPE, name.span, message);
            }
            None if prelude::is_trait(&name.text) || name.text.is_empty() => {}
            None if self.env.has_imports() => {}
            None => {
                let message = format!("no trait named `{}`", name.text);
                self.error(codes::UNKNOWN_TYPE, name.span, message);
            }
        }
        None
    }

    /// An impl of a trait written in cp must define every fn the trait
    /// leaves without a body, and no fn the trait does not have.
    fn impl_matches_trait(&mut self, trait_ty: &ast::Type, target: &ast::Type, body: &[Stmt]) {
        let Some(methods) = self.trait_exists(trait_ty) else {
            return;
        };
        let trait_text = self.quote(trait_ty.span);
        let defined: Vec<&ast::Name> = body
            .iter()
            .filter_map(|m| match &m.kind {
                StmtKind::Fn(f) => Some(&f.name),
                _ => None,
            })
            .collect();
        for name in &defined {
            if methods.iter().all(|m| m.name != name.text) {
                let known: Vec<String> = methods.iter().map(|m| m.name.clone()).collect();
                let suggestion = closest(&name.text, known.iter().map(String::as_str));
                let message = format!("`{trait_text}` has no fn `{}`", name.text);
                if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
                    match suggestion {
                        Some(s) => {
                            diag.help = Some(format!("did you mean `{s}`?"));
                            diag.fixes.push(cpl_diag::Fix {
                                title: format!("change to `{s}`"),
                                edits: vec![(name.span, s.to_string())],
                            });
                        }
                        None => {
                            diag.help =
                                Some(format!("its fns are {}", super::expr::list_names(&known)));
                        }
                    }
                }
            }
        }
        let missing: Vec<String> = methods
            .iter()
            .filter(|m| m.required && defined.iter().all(|d| d.text != m.name))
            .map(|m| m.name.clone())
            .collect();
        if !missing.is_empty() {
            let message = format!(
                "`impl {trait_text} for {}` is missing {}",
                self.quote(target.span),
                super::expr::list_names(&missing)
            );
            let span = trait_ty.span.to(target.span);
            if let Some(diag) = self.error(codes::WRONG_ARG_COUNT, span, message) {
                diag.help = Some(format!(
                    "`{trait_text}` declares {} without a body, so each impl writes {}",
                    if missing.len() == 1 { "it" } else { "them" },
                    if missing.len() == 1 { "it" } else { "them" },
                ));
            }
        }
    }

    /// Reports a `@memo` that would not remember anything. Both back ends
    /// keep one table of answers per top-level or local fn, keyed by its
    /// arguments, and run the fn as written when they can't: the program
    /// would be right and, without a word, as slow as if `@memo` were not
    /// there.
    fn memo_fits(&mut self, f: &ast::FnDecl, self_ty: Option<&Ty>, generator: bool, unit: bool, memo: Span) {
        let name = &f.name.text;
        let changed = f.params.iter().find(|p| p.mutable && p.name.text != "self");
        let (message, help) = if let Some(owner) = self_ty {
            let owner = owner.to_string();
            (
                format!("`@memo` can't remember `{name}`: it is a fn of `{owner}`"),
                format!("only a top-level or local fn keeps a table of its answers: move `{name}` out of `{owner}` and pass it what it reads"),
            )
        } else if let Some(param) = changed {
            let param = &param.name.text;
            (
                format!("`@memo` can't remember `{name}`: it changes its argument `{param}`"),
                format!("a remembered answer is given back without running the fn, so `{param}` would be left as it was: take `{param}` without `mut`, or drop `@memo`"),
            )
        } else if generator {
            (
                format!("`@memo` can't remember `{name}`: it yields its values one by one"),
                "collect the values into a list and return that: `-> List<T>`".to_string(),
            )
        } else if !f.generics.is_empty() {
            (
                format!("`@memo` can't remember the generic fn `{name}`"),
                format!("there is one table of answers per fn, and `{name}` is a different fn for each type it is called with: write it for the type it is used with"),
            )
        } else if unit {
            (
                format!("`@memo` has nothing to remember: `{name}` gives no value"),
                "a fn that only acts has to run every time: drop `@memo`, or return what it computes".to_string(),
            )
        } else {
            return;
        };
        if let Some(diag) = self.error(codes::BAD_MEMO, memo, message) {
            diag.help = Some(help);
        }
    }

    /// `memo` is where the fn's `@memo` is written, when it has one.
    fn fn_decl(&mut self, f: &ast::FnDecl, self_ty: Option<&Ty>, local: bool, memo: Option<Span>) {
        let sig = self.fn_sig(f, self_ty);
        let Ty::Fn { params, ret } = &sig else { return };
        if local {
            self.declare(&f.name.text, f.name.span, sig.clone(), false, "fn");
        } else {
            self.hovers.push(HoverEntry {
                span: f.name.span,
                name: f.name.text.clone(),
                ty: sig.clone(),
                keyword: "fn",
                decl: Some(f.name.span),
                owner: None,
            });
        }
        let outer = self.generics.len();
        self.generics
            .extend(f.generics.iter().map(|g| g.text.clone()));
        self.bounds_exist(&f.bounds);
        self.in_fn += 1;
        self.push_scope();
        // Inside a method the struct's fields can be named without `self.`.
        let receiver = f.params.iter().find(|p| p.name.text == "self");
        if let (Some(receiver), Some(Ty::Named { name, .. })) = (receiver, self_ty) {
            if let Some(Def::Struct { fields, .. }) = self.global(name).as_deref() {
                for field in fields {
                    self.scopes.last_mut().unwrap().insert(
                        field.name.clone(),
                        Local {
                            ty: field.ty.clone(),
                            mutable: receiver.mutable,
                            span: receiver.name.span,
                            keyword: if receiver.mutable { "field" } else { "self" },
                        },
                    );
                }
            }
            self.push_scope();
        }
        if let Some(lends) = &f.lends {
            self.resolve_type(&lends.ty);
        }
        let mut sig_params = params.iter();
        for param in &f.params {
            if self_ty.is_some() && param.name.text == "self" {
                let ty = self_ty.cloned().unwrap_or(Ty::Unknown);
                self.declare(
                    "self",
                    param.name.span,
                    ty,
                    param.mutable,
                    if param.mutable { "param" } else { "self" },
                );
                continue;
            }
            let ty = sig_params.next().map_or(Ty::Unknown, |p| p.ty.clone());
            if let Some(default) = &param.default {
                let default_ty = self.infer(default);
                if !self.array_fills(&ty, &default_ty, default) {
                    self.expect_fits(&default_ty, &ty, default, |c, got, want| {
                        format!(
                            "`{}` is {}, but its default `{}` is {}",
                            param.name.text,
                            want.with_article(),
                            c.quote(default.span),
                            got.with_article()
                        )
                    });
                }
            }
            self.declare(
                &param.name.text,
                param.name.span,
                ty,
                param.mutable,
                "param",
            );
        }
        // A generator's body yields its values; it has no value of its own.
        let generator = matches!(
            f.ret.as_ref().map(|t| &t.kind),
            Some(ast::TypeKind::Yields(_))
        );
        let fn_depth = self.scopes.len().saturating_sub(1);
        self.fns.push((
            f.name.text.clone(),
            (**ret).clone(),
            f.ret.is_some() && !generator,
            fn_depth,
        ));
        self.gives.push(false);
        // A fn body knows nothing of what is lent or known where the fn
        // is written: it runs when it is called.
        self.lend_isolated(|c| {
            if self_ty.is_some() {
                let fields: Vec<String> = match self_ty {
                    Some(Ty::Named { name, .. }) => c
                        .members_of(name)
                        .map(|m| m.fields.iter().map(|f| f.name.clone()).collect())
                        .unwrap_or_default(),
                    _ => Vec::new(),
                };
                c.projection_shape(f, &fields);
            }
            match &f.body {
                FnBody::Block(block) => {
                    c.push_scope();
                    let mark = c.lend_mark();
                    let ty = c.stmts(&block.stmts, false);
                    // The last expression is the fn's value. Check returns before
                    // popping the fn body scope so local domain declarations are
                    // still visible for escape checking.
                    if let Some(Stmt {
                        kind: StmtKind::Expr(last),
                        ..
                    }) = block.stmts.last()
                    {
                        // An `if` or `match` whose branches all `return` has no
                        // value; the returns were checked on their own.
                        let statement_like = ty == Ty::Unit
                            && matches!(last.kind, ExprKind::If { .. } | ExprKind::Match { .. });
                        if !statement_like {
                            c.returns(last, &ty);
                            c.no_escape(last, "returning it");
                        }
                    }
                    c.lend_reset(mark);
                    c.pop_scope();
                }
                FnBody::Expr(expr) => {
                    if matches!(expr.kind, ExprKind::If { .. } | ExprKind::Match { .. }) {
                        c.plain_if = Some(expr.span);
                    }
                    let ty = c.infer(expr);
                    c.returns(expr, &ty);
                    c.no_escape(expr, "returning it");
                }
                FnBody::None => {}
            }
        });
        self.fns.pop();
        let gives = self.gives.pop().unwrap_or(false);
        if let Some(memo) = memo {
            // Written `-> T`, or read off what the body gives.
            let unit = match &f.ret {
                Some(_) => **ret == Ty::Unit,
                None => !gives && !matches!(f.body, FnBody::None),
            };
            self.memo_fits(f, self_ty, generator, unit, memo);
        }
        self.pop_scope();
        if receiver.is_some() && matches!(self_ty, Some(Ty::Named { .. })) {
            self.pop_scope();
        }
        self.in_fn -= 1;
        self.generics.truncate(outer);
    }
}

/// Where a fn's `@memo` is written.
fn memo_of(stmt: &Stmt) -> Option<Span> {
    let memo = stmt.decorators.iter().find(|d| d.name.text == "memo")?;
    Some(memo.span)
}

/// The span of each name a pattern binds, in the order `pattern_types`
/// visits them.
fn name_spans(pat: &Pattern, out: &mut Vec<Span>) {
    match &pat.kind {
        PatternKind::Name(_) => out.push(pat.span),
        PatternKind::Domain(name) => out.push(name.span),
        PatternKind::Tuple(items) => {
            for item in items {
                match &item.kind {
                    PatternKind::Domain(name) => out.push(name.span),
                    _ => match super::types::domain_binding(item) {
                        Some(_) => out.push(item.span),
                        None => name_spans(item, out),
                    },
                }
            }
        }
        _ => {}
    }
}

impl Checker<'_> {
    /// The values of `fields(a, b, ..)` in a `for` head: the compile-time
    /// loop over a struct's or a tuple's fields (decision 28.1).
    pub(crate) fn reflected_values<'e>(&mut self, iter: &'e Expr) -> Option<&'e [cpl_syntax::ast::Arg]> {
        let ExprKind::Call { callee, args } = &iter.kind else {
            return None;
        };
        let ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        (name == "fields"
            && self.local(name).is_none()
            && self.global(name).is_none()
            && !args.is_empty()
            && args.iter().all(|a| a.name.is_none()))
        .then_some(args.as_slice())
    }

    /// The fields of a value of type `ty`, by name, in declaration order;
    /// `Ok(None)` when the type is not known until the code is instantiated.
    fn variant_payloads(&mut self, ty: &Ty) -> Option<Vec<Vec<Ty>>> {
        let Ty::Named { name, .. } = ty else {
            return None;
        };
        match self.global(name).as_deref() {
            Some(Def::Enum { variants, .. }) => Some(variants.iter().map(|(_, fs)| fs.clone()).collect()),
            _ => None,
        }
    }

    pub(crate) fn reflect(&mut self, ty: &Ty) -> Result<Option<Vec<(String, Ty)>>, ()> {
        match ty {
            Ty::Tuple(items) => Ok(Some(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, t)| (i.to_string(), t.clone()))
                    .collect(),
            )),
            Ty::Record(fields) => Ok(Some(fields.clone())),
            Ty::Named { name, args } => {
                let Some(members) = self.members_of(name) else {
                    return Ok(None);
                };
                if members.fields.is_empty() {
                    return Err(());
                }
                let bindings = members.bindings(args);
                let fields: Vec<_> = members
                    .fields
                    .iter()
                    .map(|f| (f.name.clone(), f.ty.subst(&bindings)))
                    .collect();
                Ok(Some(
                    fields
                        .into_iter()
                        .map(|(n, t)| (n, self.settle(&t)))
                        .collect(),
                ))
            }
            Ty::Param(_) | Ty::Unknown | Ty::Error => Ok(None),
            ty if ty.is_vague() => Ok(None),
            _ => Err(()),
        }
    }

    /// `for f in fields(x):` is checked once per field, with `f` of that
    /// field's type, as the back ends unroll it; inside a generic body the
    /// field types are not known yet and `f` is checked once, untyped.
    fn fields_loop(&mut self, pat: &Pattern, iter: &Expr, body: &cpl_syntax::ast::Block) {
        let values = self.reflected_values(iter).unwrap_or_default();
        let mut variant_shapes: Option<Vec<Vec<Ty>>> = None;
        let mut shapes = Vec::new();
        let mut first: Option<Ty> = None;
        for v in values {
            let ty = self.infer(&v.value);
            if let Some(first) = &first {
                if !first.is_vague() && !ty.is_vague() && *first != ty {
                    self.error(
                        codes::TYPE_MISMATCH,
                        v.value.span,
                        format!(
                            "`fields` walks values of one type together; this is a `{ty}`, the first is a `{first}`"
                        ),
                    );
                }
            } else {
                first = Some(ty.clone());
            }
            if let Some(variants) = self.variant_payloads(&ty) {
                variant_shapes = Some(variants);
                shapes.push(None);
                continue;
            }
            match self.reflect(&ty) {
                Ok(fields) => shapes.push(fields),
                Err(()) => {
                    if let Some(d) = self.error(
                        codes::TYPE_MISMATCH,
                        v.value.span,
                        format!("`fields` walks the fields of a struct or a tuple, not of a `{ty}`"),
                    ) {
                        d.help = Some("pass a struct or tuple value: `for f in fields(p):`".to_string());
                    }
                    shapes.push(None);
                }
            }
        }
        if let Some(stmt) = loop_exit(&body.stmts) {
            self.error(
                codes::UNSUPPORTED,
                stmt,
                "a `fields` loop runs its body once per field at compile time, so it has no `break` or `continue`".to_string(),
            );
        }
        let elem = |i: Option<usize>, shapes: &[Option<Vec<(String, Ty)>>]| -> Ty {
            let one = |s: &Option<Vec<(String, Ty)>>| match (i, s) {
                (Some(i), Some(f)) => f.get(i).map(|(_, t)| t.clone()).unwrap_or(Ty::Unknown),
                _ => Ty::Unknown,
            };
            if shapes.len() == 1 {
                one(&shapes[0])
            } else {
                Ty::Tuple(shapes.iter().map(one).collect())
            }
        };
        // An enum value walks the payload of the variant it is: the body is
        // checked for each variant's fields.
        if let Some(variants) = variant_shapes {
            let n = shapes.len();
            for fields in variants {
                for ty in fields {
                    let ty = if n == 1 { ty } else { Ty::Tuple(vec![ty; n]) };
                    self.push_scope();
                    self.bind(pat, &ty, false, "for");
                    self.block(body);
                    self.pop_scope();
                }
            }
            return;
        }
        let count = shapes.first().and_then(|s| s.as_ref().map(Vec::len));
        let passes: Vec<Option<usize>> = match count {
            Some(n) => (0..n).map(Some).collect(),
            None => vec![None],
        };
        for i in passes {
            let ty = elem(i, &shapes);
            self.push_scope();
            self.bind(pat, &ty, false, "for");
            self.block(body);
            self.pop_scope();
        }
    }
}

/// The span of a `break` or `continue` that leaves this loop, outside any
/// loop nested in it.
fn loop_exit(stmts: &[Stmt]) -> Option<Span> {
    stmts.iter().find_map(|s| match &s.kind {
        StmtKind::Break(None) | StmtKind::Continue(None) => Some(s.span),
        StmtKind::Expr(Expr {
            kind: ExprKind::If { branches, otherwise },
            ..
        }) => branches
            .iter()
            .map(|(_, b)| b)
            .chain(otherwise)
            .find_map(|b| loop_exit(&b.stmts)),
        _ => None,
    })
}

fn refutable(pat: &ast::Pattern) -> bool {
    match &pat.kind {
        ast::PatternKind::Variant { .. } | ast::PatternKind::Literal(_) => true,
        ast::PatternKind::Tuple(items) => items.iter().any(refutable),
        ast::PatternKind::Or(alts) => alts.iter().all(refutable),
        _ => false,
    }
}

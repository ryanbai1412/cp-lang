//! Statements, blocks, and declarations.

mod fn_decl;
mod part_assign;
mod reflect;
mod traits;

use super::pattern::{name_spans, pattern_types};
use super::*;
use cpl_diag::msgs;
use cpl_syntax::ast::{
    self, AssignOp, Block, Expr, ExprKind, FnBody, Pattern, PatternKind, Stmt, StmtKind,
};
use fn_decl::memo_of;

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

    /// Each variant reads and prints as its `@tag`, or else its name, so
    /// two variants may not share one (R02.7.2).
    /// The range a constant expression is known to lie in: literals,
    /// constant `let`s and `+ - *` on them (R02.10.2).
    pub(crate) fn const_range(&self, expr: &Expr) -> Option<(i128, i128)> {
        use cpl_syntax::ast::{BinOp, UnOp};
        match &expr.kind {
            ExprKind::Int(v) => Some((*v, *v)),
            ExprKind::Paren(e) => self.const_range(e),
            ExprKind::Unary {
                op: UnOp::Neg,
                expr,
            } => self.const_range(expr).map(|(lo, hi)| (-hi, -lo)),
            ExprKind::Name(n) => match self.local(n) {
                Some(local) if local.keyword == "let" => self.let_ranges.get(n).copied(),
                Some(_) => None,
                None => self.let_ranges.get(n).copied(),
            },
            ExprKind::Binary { op, lhs, rhs, .. } => {
                let (a, b) = (self.const_range(lhs)?, self.const_range(rhs)?);
                match op {
                    BinOp::Add => Some((a.0.checked_add(b.0)?, a.1.checked_add(b.1)?)),
                    BinOp::Sub => Some((a.0.checked_sub(b.1)?, a.1.checked_sub(b.0)?)),
                    BinOp::Mul => {
                        let c = [
                            a.0.checked_mul(b.0)?,
                            a.0.checked_mul(b.1)?,
                            a.1.checked_mul(b.0)?,
                            a.1.checked_mul(b.1)?,
                        ];
                        Some((*c.iter().min()?, *c.iter().max()?))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// R06.4.8: dropping what one of the file's fns returns is W0101
    /// unless the fn is `@droppable`.
    pub(super) fn dropped_result(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, .. } = &expr.kind else {
            return;
        };
        let ExprKind::Name(name) = &callee.kind else {
            return;
        };
        if self.local(name).is_some() || !self.env.names().contains(name) {
            return;
        }
        let Some(def) = self.free_global(name) else {
            return;
        };
        let Def::Fn {
            ty: Ty::Fn { ret, .. },
            droppable: false,
            ..
        } = &*def
        else {
            return;
        };
        if matches!(**ret, Ty::Unit | Ty::Error | Ty::Unknown) {
            return;
        }
        let shown = self.quote(expr.span);
        self.sink.push(
            cpl_diag::msgs::check::RESULT_DROPPED
                .warning(expr.span, &[&shown, &ret.with_article()])
                .with_help(format!(
                "use it (`let r = {shown}`), or mark `{name}` `@droppable` if its result is optional"
            )),
        );
    }

    /// R03.1.4: storing a value proven outside `lo..=hi` is an error.
    fn range_store(&mut self, value: &Expr, target: &Ty) {
        let Ty::Range { lo, hi, text } = target else {
            return;
        };
        let Some((vlo, vhi)) = self.const_range(value) else {
            return;
        };
        let below = lo.is_some_and(|lo| vhi < lo);
        let above = hi.is_some_and(|hi| vlo > hi);
        if !(below || above) {
            return;
        }
        let shown = self.quote(value.span);
        let is = if vlo == vhi {
            format!("{vlo}")
        } else {
            format!("in {vlo}..={vhi}")
        };
        let message = if shown == is {
            cpl_diag::msgs::check::OUTSIDE_RANGE_TYPE.at(&[&shown, &text])
        } else {
            cpl_diag::msgs::check::OUTSIDE_RANGE_TYPE_2.at(&[&shown, &is, &text])
        };
        let text = text.clone();
        if let Some(diag) = self.report_said(value.span, message) {
            diag.help = Some(format!(
                "store a value in `{text}`, or widen the declared type"
            ));
        }
    }

    /// 0.3 writes `enum Dir = N | E`; `enum Dir: N, E` is gone.
    pub(super) fn no_comma_variants(&mut self, variants: &[cpl_syntax::ast::Variant]) {
        for pair in variants.windows(2) {
            let (a, b) = (&pair[0].name, &pair[1].name);
            if !pair[0].fields.is_empty() || pair[0].value.is_some() || pair[1].tag.is_some() {
                continue;
            }
            let between = self.text(a.span.to(b.span));
            let gap = &between
                [a.text.len().min(between.len())..between.len().saturating_sub(b.text.len())];
            if gap.trim() != "," {
                continue;
            }
            let names: Vec<&str> = variants.iter().map(|v| v.name.text.as_str()).collect();
            if let Some(diag) = self.report(
                a.span.to(b.span),
                msgs::check::EXPECTED_TOKEN_AN_ENUM_S_VARIANTS,
                &[],
            ) {
                diag.help = Some(format!("write `= {}`", names.join(" | ")));
            }
            return;
        }
    }

    /// `var x = None` and `let n = read()` give no type, and inference
    /// never looks at later statements (R02.10.1, R10.2.1).
    fn needs_annotation(&mut self, keyword: &str, pat: &Pattern, init: Option<&Expr>) -> bool {
        let (PatternKind::Name(name), Some(init)) = (&pat.kind, init) else {
            return false;
        };
        let (message, help) = match &init.kind {
            ExprKind::Call { callee, args }
                if args.is_empty()
                    && matches!(&callee.kind, ExprKind::Name(n) if n == "read")
                    && self.local("read").is_none() =>
            {
                (
                    cpl_diag::msgs::check::READ_DOESNT_SAY_WHAT.at(&[&keyword, &name]),
                    format!("annotate it: `{keyword} {name}: Int = read()`"),
                )
            }
            _ => return false,
        };
        if let Some(diag) = self.report_said(init.span, message) {
            diag.help = Some(help);
        }
        true
    }

    /// R01.3.2: a `let`/`var` name is lowercase; `N` would read as a type.
    fn value_case(
        &mut self,
        keyword: &str,
        pat: &Pattern,
        init: Option<&Expr>,
        init_ty: Option<&Ty>,
    ) {
        if matches!(init_ty, Some(Ty::Domain(_) | Ty::Error)) {
            return;
        }
        let name = match &pat.kind {
            PatternKind::Name(name) => name.as_str(),
            PatternKind::Variant { path, args } if path.len() == 1 && args.is_empty() => {
                path[0].text.as_str()
            }
            _ => return,
        };
        if !name.starts_with(|c: char| c.is_ascii_uppercase()) {
            return;
        }
        let lower = name.to_lowercase();
        let message =
            cpl_diag::msgs::check::VALUES_ARE_LOWERCASE_WOULD.at(&[&keyword, &name, &name]);
        let fix = self.const_fix(keyword, pat.span, init);
        if let Some(diag) = self.report_said(pat.span, message) {
            diag.help = Some(format!("rename it `{lower}`, or write `const {name}`"));
            diag.fixes.extend(fix);
        }
    }

    fn array_fills(&mut self, declared: &Ty, init_ty: &Ty, init: &Expr) -> bool {
        // `read()` reads the whole array, not one cell.
        if let ExprKind::Call { callee, args } = &init.kind {
            if args.is_empty() && matches!(&callee.kind, ExprKind::Name(n) if n == "read") {
                return false;
            }
        }
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

    /// Checks statements that run one after another. A slice or row bound by `let` keeps its
    /// array lent until the last statement that names it.
    fn stmts(&mut self, stmts: &[Stmt], top_level: bool) -> Ty {
        let mut last = Ty::Unit;
        let present = self.present.len();
        let memo_captured = self.memo_captured.len();
        let local_fn_captures = self.local_fn_captures.len();
        if !top_level {
            self.declare_local_types(stmts);
            let mut sibling_vars: Vec<String> = Vec::new();
            for stmt in stmts {
                if let StmtKind::Let { pat, .. } = &stmt.kind {
                    pattern_types(pat, &Ty::Unknown, &mut |name, _| {
                        sibling_vars.push(name.to_string());
                    });
                }
            }
            for stmt in stmts {
                if let StmtKind::Fn(f) = &stmt.kind {
                    let sig = self.quietly(|c| c.fn_sig(f, None));
                    self.declare(&f.name.text, f.name.span, sig, false, "fn");
                    let eff = super::lends::own_effects(f);
                    let caps: Vec<String> = eff
                        .reads
                        .into_iter()
                        .chain(eff.writes)
                        .filter(|n| sibling_vars.contains(n))
                        .collect();
                    if !caps.is_empty() {
                        self.local_fn_captures.push((f.name.text.clone(), caps));
                    }
                }
            }
        }
        let mut leading = true;
        for stmt in stmts {
            match &stmt.kind {
                StmtKind::With { body: None, .. } if !leading => self.with_not_first(stmt.span),
                StmtKind::With { body: None, .. } => {}
                _ if runs(stmt) => leading = false,
                _ => {}
            }
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
            match &stmt.kind {
                StmtKind::Let {
                    mutable: true,
                    pat:
                        Pattern {
                            kind: PatternKind::Name(name),
                            ..
                        },
                    init: Some(value),
                    ..
                } => self.begin_var_hold(stmt, name, value, &stmts[k + 1..]),
                StmtKind::Assign {
                    target,
                    op: AssignOp::Set,
                    value,
                } => {
                    if let ExprKind::Name(name) = &super::lends::unparen(target).kind {
                        // The store replaces the value, and with it the loan it held.
                        self.lends.holds.retain(|h| h.is_capture || h.name != *name);
                        self.begin_var_hold(stmt, name.as_str(), value, &stmts[k + 1..]);
                    }
                }
                _ => {}
            }
            self.end_holds(stmt);
        }
        self.present.truncate(present);
        self.memo_captured.truncate(memo_captured);
        self.local_fn_captures.truncate(local_fn_captures);
        last
    }

    /// Binds the names in `pat` to parts of a value of type `ty`.
    pub(crate) fn bind(&mut self, pat: &Pattern, ty: &Ty, mutable: bool, keyword: &'static str) {
        match &pat.kind {
            PatternKind::Tuple(_) if refutable(pat) => {
                self.bind_refutable(pat, ty, mutable, keyword)
            }
            PatternKind::Name(_)
            | PatternKind::Domain(_)
            | PatternKind::Tuple(_)
            | PatternKind::Record { .. } => {
                let effective_ty;
                let bind_ty = if matches!(pat.kind, PatternKind::Tuple(_)) {
                    if let Ty::Pos(dom) = ty {
                        if let Some(factors) = self.product_factors(dom) {
                            effective_ty = Ty::Tuple(factors.into_iter().map(|(_, t)| t).collect());
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
                pattern_types(pat, bind_ty, &mut |name, ty| {
                    names.push((name.to_string(), ty))
                });
                let mut spans = Vec::new();
                name_spans(pat, false, &mut spans);
                for ((name, ty), (span, is_var)) in names.into_iter().zip(spans) {
                    let is_mut = mutable || is_var;
                    let kw = if matches!(ty, Ty::Domain(_)) {
                        "domain"
                    } else if is_var {
                        "var"
                    } else {
                        keyword
                    };
                    self.declare(&name, span, ty, is_mut, kw);
                }
            }
            PatternKind::Variant { path, args } => {
                if let Some(name) = path.last() {
                    if let Some(def) = self.global(&name.text) {
                        self.hovers.push(HoverEntry {
                            span: name.span,
                            name: name.text.clone(),
                            ty: def.value_ty(&name.text),
                            keyword: "global",
                            decl: None,
                            owner: None,
                        });
                    }
                }
                let fields = self.variant_payload(path, ty);
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
            PatternKind::Var(inner) => self.bind(inner, ty, true, "var"),
            PatternKind::Wildcard | PatternKind::Error => {}
        }
    }

    /// What `let pat = init else: ...` binds: the value in each optional. A tuple of values is
    /// taken apart, `let (x, y) = (a, b) else: ...`.
    /// A refutable pattern, `let (gap, Some(best)) = e else: ...`, is matched
    /// against `e` itself.
    pub(crate) fn let_else(&mut self, pat: &ast::Pattern, init: &ast::Expr, ty: Ty) -> Ty {
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
                        self.report(
                            item.span,
                            msgs::check::TYPE_MISMATCH_LET_ELSE_NEEDS_A,
                            &[&self.quote(item.span), &ty.with_article()],
                        );
                        Ty::Error
                    }));
                }
                Ty::Tuple(out)
            }
            (ast::PatternKind::Name(_) | ast::PatternKind::Tuple(_), _, ty) => inside(ty)
                .unwrap_or_else(|ty| {
                    self.report(
                        init.span,
                        msgs::check::TYPE_MISMATCH_LET_ELSE_NEEDS_A_2,
                        &[&self.quote(init.span), &ty.with_article()],
                    );
                    Ty::Error
                }),
            (_, _, ty) => inside(ty).unwrap_or(Ty::Unknown),
        }
    }

    pub(super) fn stmt_body(&mut self, stmt: &Stmt, top_level: bool) -> Ty {
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
                let declared = self.let_declared(pat, ty.as_ref(), init.as_ref());
                if let (Some(ast_ty), Some(init_expr)) = (ty, init) {
                    if let Some(alphabet) = ast_ty.kind.alphabet() {
                        if let ExprKind::Str(pieces) = &init_expr.kind {
                            for piece in pieces {
                                if let cpl_syntax::ast::StrPiece::Text(text) = piece {
                                    if let Some(bad) = text.chars().find(|c| !alphabet.contains(*c))
                                    {
                                        self.report(
                                            init_expr.span,
                                            msgs::check::RANGE_STORE_CONST_IS_NOT_IN_STR,
                                            &[&bad, &alphabet],
                                        );
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if let (ast::TypeKind::LiteralUnion(lits), PatternKind::Name(n)) =
                        (&ast_ty.kind, &pat.kind)
                    {
                        self.lit_unions.insert(n.clone(), lits.clone());
                    }
                }
                let prev_bounds_range = self.expected_bounds_range;
                if declared.as_ref().is_some_and(Self::expects_bounds_range) {
                    self.expected_bounds_range = true;
                }
                let prev_piece = std::mem::replace(
                    &mut self.expected_piece,
                    declared.as_ref().is_some_and(Self::expects_piece),
                );
                if let (
                    Some(declared),
                    Some(Expr {
                        kind: ExprKind::Call { callee, args },
                        ..
                    }),
                ) = (&declared, init)
                {
                    match &callee.kind {
                        ExprKind::Name(n)
                            if n == "read" && args.is_empty() && self.io_fixed(declared) =>
                        {
                            self.fixed_io.push(callee.span)
                        }
                        ExprKind::Name(n) if n == "ask" && self.io_fixed(declared) => {
                            self.record_io(callee.span, n, args);
                        }
                        _ => {}
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
                                .map(|before| {
                                    Span::new(before.len() as u32, before.len() as u32 + 3)
                                });
                            if let Some(diag) = self.report(
                                at,
                                msgs::check::VAR_LAMBDA_RECURSION_VAR_CANNOT_REFER_TO,
                                &[&vname],
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
                let unannotated =
                    declared.is_none() && self.needs_annotation(keyword, pat, init.as_ref());
                let mut init_ty = init.as_ref().map(|init| self.infer(init));
                if unannotated {
                    init_ty = Some(Ty::Error);
                }
                self.value_case(keyword, pat, init.as_ref(), init_ty.as_ref());
                if let PatternKind::Name(name) = &pat.kind {
                    let known = match (keyword, &declared, init.as_ref()) {
                        ("let", None, Some(init)) => self.const_range(init),
                        _ => None,
                    };
                    match known {
                        Some(r) => self.let_ranges.insert(name.clone(), r),
                        None => self.let_ranges.remove(name),
                    };
                }
                if let (Some(declared), Some(init)) = (&declared, init.as_ref()) {
                    self.range_store(init, declared);
                }
                if let (Some(declared), Some(init)) = (&declared, init.as_ref()) {
                    self.fold_needs_identity(init, declared);
                }
                if let PatternKind::Name(name) = &pat.kind {
                    match init.as_ref() {
                        Some(init) if keyword == "let" && self.walk_only_expr(init) => {
                            self.walk_only.insert(name.clone());
                        }
                        _ => {
                            self.walk_only.remove(name);
                        }
                    }
                }
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
                    if !otherwise.leaves() {
                        if let Some(diag) = self.report(
                            pat.span,
                            msgs::check::ELSE_FALLS_THROUGH_THE_ELSE_BLOCK_OF,
                            &[],
                        ) {
                            diag.help = Some("end it with `return`, `break`, `continue` or a trap such as `todo`".to_string());
                        }
                    }
                }
                // `var seen: [V]Bool = false` fills the array with the value.
                // A comprehension builds the array when one is declared.
                let fills = match (&declared, &init_ty, init) {
                    (Some(decl), Some(ity), Some(init_expr)) => {
                        self.array_fills(decl, ity, init_expr)
                    }
                    _ => false,
                };
                // `var adj: [V]List<V> = []`: the `[]` is one cell.
                if let (true, Some(Ty::Array { elem, .. }), Some(init)) = (fills, &declared, init) {
                    if matches!(&init.kind, ExprKind::List(items) if items.is_empty()) {
                        self.note_read(init, elem);
                    }
                }
                let view = match (&declared, init) {
                    (Some(Ty::Named { name, .. }), Some(init)) if name == "List" => {
                        self.view_kind(init)
                    }
                    _ => None,
                };
                if let (Some(declared), Some(init), Some(kind)) = (&declared, init, view) {
                    let kind = if kind == "slice" { "slice view" } else { kind };
                    let init_text = self.quote(init.span);
                    if let Some(diag) = self.report(
                        init.span,
                        msgs::check::TYPE_MISMATCH_IS_DECLARED_BUT_IS,
                        &[
                            &self.quote(pat.span),
                            &declared.with_article(),
                            &init_text,
                            &kind,
                        ],
                    ) {
                        diag.help = Some(format!(
                            "a view is lent from its source; copy it with `List({init_text})` (R07.2.11)"
                        ));
                        if kind == "slice view" {
                            diag.fixes.push(cpl_diag::Fix {
                                title: "copy the view into a list".to_string(),
                                edits: vec![(init.span, format!("List({init_text})"))],
                            });
                        }
                    }
                } else if let (Some(declared), Some(init_ty), Some(init), false) =
                    (&declared, &init_ty, init, fills)
                {
                    let name = self.quote(pat.span);
                    // The annotation widens a fresh value's range cells (decision 366).
                    let fresh = !matches!(
                        init.kind,
                        ExprKind::Name(_) | ExprKind::Field { .. } | ExprKind::Index { .. }
                    );
                    let message = |c: &Self, got: &Ty, want: &Ty| {
                        cpl_diag::msgs::check::DECLARED_BUT.at(&[
                            &name,
                            &want.with_article(),
                            &c.quote(init.span),
                            &got.with_article(),
                        ])
                    };
                    if fresh {
                        self.expect_fits(init_ty, declared, init, message);
                    } else {
                        self.expect_moves(init_ty, declared, init, message);
                    }
                }
                let guessed = declared.is_none()
                    && self.sink.encloses_poison(stmt.span)
                    && crate::db::head_leaves_open(self.src, stmt);
                self.empty_braces(declared.as_ref(), init.as_ref());
                if declared.is_none() && !guessed {
                    let is_bare_none = matches!(
                        init,
                        Some(Expr {
                            kind: ExprKind::None,
                            ..
                        })
                    );
                    let is_bare_read = matches!(
                        init,
                        Some(Expr {
                            kind: ExprKind::Call { callee, args },
                            ..
                        }) if args.is_empty()
                            && matches!(&callee.kind, ExprKind::Name(n) if n == "read")
                    ) && matches!(init_ty.as_ref(), Some(Ty::Unknown));
                    if is_bare_none {
                        let name = self.quote(pat.span);
                        if let Some(diag) = self.report(
                            pat.span,
                            msgs::check::INFERENCE_NEEDS_ANNOTATION_NEEDS_A_TYPE_ANNOTATION,
                            &[&name],
                        ) {
                            diag.help = Some(format!("write `{keyword} {name}: ?Int = None`"));
                        }
                    } else if is_bare_read {
                        let name = self.quote(pat.span);
                        let message = cpl_diag::msgs::check::READ_NEEDS_TYPE_ANNOTATION.at(&[]);
                        if let Some(diag) = self.report_said(pat.span, message) {
                            diag.help = Some(format!("write `{keyword} {name}: Int = read()`"));
                        }
                    }
                }
                let ty = if guessed {
                    Ty::Error
                } else if declared.is_none()
                    && *mutable
                    && matches!(init_ty, Some(Ty::Range { .. }))
                {
                    Ty::Int
                } else if declared.is_none() && !*mutable && init_ty.as_ref() == Some(&Ty::Int) {
                    if let Some(init_expr) = init {
                        let (lo, hi) = self.bounds(init_expr);
                        if let (Some(l), Some(h)) = (lo, hi) {
                            Ty::Range {
                                lo: Some(l),
                                hi: Some(h),
                                text: format!("{l}..={h}"),
                            }
                        } else {
                            Ty::Int
                        }
                    } else {
                        Ty::Int
                    }
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
                        let src_is_heap = init_ty.as_ref().is_some_and(|t| self.is_heap_type(t));
                        if is_owned && src_is_heap {
                            self.mark_moved(src, *src_span);
                        }
                        let is_param = self.local(src).is_some_and(|l| l.keyword == "param");
                        // A `var xs` parameter opens its body with this binding, spanning the name.
                        let var_param = pat.span == *src_span;
                        if *mutable && is_param && src_is_heap && !var_param {
                            let name = self.quote(pat.span);
                            let what = format!("var {name}");
                            if let Some(diag) = self.report(
                                *src_span,
                                msgs::check::USE_AFTER_MOVE_VAR_FROM_LENT_PARAM,
                                &[&what, src],
                            ) {
                                diag.help = Some(format!(
                                    "keep your own copy: `var {name} = {src}.clone()`"
                                ));
                            }
                        }
                    }
                }
                if let Some(init) = init {
                    let what = format!("`{} {}`", binding_kw(*mutable), self.quote(pat.span));
                    self.escape_unless_kept(init, &what, Keep::binding(*mutable, pat.span));
                }
                // A new binding of a name: what was known about the old
                // one no longer applies.
                let mut bound = Vec::new();
                pattern_types(pat, &ty, &mut |name, _| bound.push(name.to_string()));
                self.drop_facts(&bound);
                if let (
                    ast::PatternKind::Name(n),
                    Some(Expr {
                        kind: ExprKind::Lambda { params, body, .. },
                        ..
                    }),
                ) = (&pat.kind, init)
                {
                    self.local_effects
                        .insert(n.clone(), super::lends::lambda_effects(params, body));
                }
                let is_mut = *mutable
                    || matches!(
                        init,
                        Some(Expr {
                            kind: ExprKind::Mut(_),
                            ..
                        })
                    );
                match super::pattern::domain_binding(pat) {
                    // `let LIM = 100`: with no `else` there is no variant
                    // to match, so a capitalized name is bound to the value.
                    Some(name)
                        if otherwise.is_none()
                            && !matches!(
                                self.global(name).as_deref(),
                                Some(Def::Variant { .. })
                            ) =>
                    {
                        self.report(
                            pat.span,
                            msgs::check::VALUE_NAME_CASE_VALUE_STARTS_WITH_AN,
                            &[&name, &name],
                        );

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
                        cpl_diag::msgs::check::DECLARED_BUT.at(&[
                            &name.text,
                            &want.with_article(),
                            &c.quote(init.span),
                            &got.with_article(),
                        ])
                    });
                }
                let ty = declared.or(init_ty).unwrap_or(Ty::Error);
                self.generics.truncate(outer);
                self.declare(&name.text, name.span, ty, false, "const");
            }
            StmtKind::Fn(f) => {
                // `@memo(into: dp)` on a fn inside a fn or a loop: the
                // table is a `var` where the fn stands.
                if let (false, Some((table, at))) = (top_level, stmt.memo_into()) {
                    if let Ty::Fn { params, ret } = self.fn_sig(f, None) {
                        let ty = Ty::Array {
                            index: params.iter().map(|p| p.ty.clone()).collect(),
                            elem: Box::new(Ty::Optional(Box::new(ret.widen()))),
                        };
                        self.declare(table, at, ty, true, "var");
                    }
                }
                let memo_info = memo_of(stmt);
                if let (false, Some((memo_span, _))) = (top_level, memo_info) {
                    let captured = self.memo_captured_vars(f);
                    for var_name in captured {
                        self.memo_captured
                            .push((var_name, f.name.text.clone(), memo_span));
                    }
                }
                self.fn_decl(f, None, !top_level, memo_info)
            }
            StmtKind::Struct(s) => {
                let generics: Vec<String> = s.generics.iter().map(|g| g.text.clone()).collect();
                let self_ty = Ty::Named {
                    name: s.name.text.clone(),
                    args: generics.iter().cloned().map(Ty::Param).collect(),
                };
                let body = self.enter_body(&generics, self_ty.clone());
                self.struct_decl_rules(s);
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
                                cpl_diag::msgs::check::BUT_DEFAULT.at(&[
                                    &field.name.text,
                                    &want.with_article(),
                                    &c.quote(default.span),
                                    &got.with_article(),
                                ])
                            });
                        }
                    }
                }
                self.type_hover(&s.name);
                let prev_cached = std::mem::replace(
                    &mut self.cached_fields,
                    s.fields
                        .iter()
                        .filter(|f| f.cached)
                        .map(|f| f.name.text.clone())
                        .collect(),
                );
                self.bounded_methods(&s.methods, &self_ty, &s.bounds);
                self.cached_fields = prev_cached;
                self.leave_body(body);
            }
            StmtKind::Enum(e) => {
                let generics: Vec<String> = e.generics.iter().map(|g| g.text.clone()).collect();
                let self_ty = Ty::Named {
                    name: e.name.text.clone(),
                    args: generics.iter().cloned().map(Ty::Param).collect(),
                };
                let body = self.enter_body(&generics, self_ty.clone());
                self.enum_decl_rules(e);
                self.enum_tags(stmt, e);
                for variant in &e.variants {
                    let fields: Vec<Ty> = variant
                        .fields
                        .iter()
                        .map(|(_, ty)| self.resolve_type(ty))
                        .collect();
                    let ty = Def::variant_value(&e.name.text, &fields);
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
                if !self.domain_sum(def) {
                    self.infer(def);
                }
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
            StmtKind::TypeAlias {
                name, generics, ty, ..
            } => {
                let aliased = self.resolve_alias(name, generics, ty);
                self.type_hover(name);
                if !top_level {
                    self.scopes.innermost().insert(
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
                ..
            } => {
                let mut generics: Vec<String> = generics.iter().map(|g| g.text.clone()).collect();
                for stmt in &body.stmts {
                    if let StmtKind::TypeAlias { name, .. } = &stmt.kind {
                        generics.push(name.text.clone());
                    }
                }
                let self_ty = Ty::Param("Self".into());
                let scope = self.enter_body(&generics, self_ty.clone());
                let outer_bounds = self.param_bounds.len();
                self.param_bounds
                    .push(("Self".into(), vec![name.text.clone()]));
                for sup in supers {
                    self.trait_exists(sup);
                }
                self.type_hover(name);
                self.owned(&name.text, &body.stmts);
                self.own_fns_scope(&name.text, &body.stmts);
                // A default body is written for the trait's own `type Fold = ?T` (decision 469).
                let defaults: Vec<(String, Ty)> = super::assoc::declared(&body.stmts)
                    .into_iter()
                    .filter_map(|(n, _, d)| d.map(|d| (n, self.quietly(|c| c.resolve_type(d)))))
                    .collect();
                let outer_assoc = std::mem::replace(&mut self.self_assoc, defaults);
                self.methods(&body.stmts, &self_ty);
                self.self_assoc = outer_assoc;
                self.pop_scope();
                self.param_bounds.truncate(outer_bounds);
                self.leave_body(scope);
            }
            StmtKind::Impl {
                trait_name,
                target,
                bounds,
                body,
            } => {
                let mut generics = self.impl_generics(trait_name.as_ref(), target, bounds);
                for stmt in &body.stmts {
                    if let StmtKind::TypeAlias { name, .. } = &stmt.kind {
                        generics.push(name.text.clone());
                    }
                }
                let scope = self.enter_body(&generics, Ty::Unknown);
                let self_ty = self.resolve_type(target);
                self.self_ty = Some(self_ty.clone());
                self.impl_home(trait_name.as_ref(), target, &self_ty);
                self.bounds_exist(bounds);
                if let Some(tr) = trait_name {
                    let file_names = self.env.declared();
                    let target_head = type_head(target);
                    let trait_head = type_head(tr);
                    let target_local = match &target.kind {
                        ast::TypeKind::ItemOf(item) => {
                            item.fn_item_key().is_some_and(|k| file_names.contains(&k))
                        }
                        _ => target_head.as_ref().is_some_and(|t| file_names.contains(t)),
                    };
                    let trait_local = trait_head.as_ref().is_some_and(|t| file_names.contains(t));
                    if !target_local
                        && !trait_local
                        && !matches!(&self_ty, Ty::Param(_) | Ty::Unknown | Ty::Error)
                    {
                        self.report(
                            tr.span,
                            msgs::check::UNSUPPORTED_ORPHAN_IMPL_NEITHER_NOR,
                            &[&self.quote(tr.span), &self.quote(target.span)],
                        );
                    }
                }

                if let Some(trait_name) = trait_name {
                    self.impl_supertraits(trait_name, target, bounds);
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
                    self.scopes.innermost().insert(
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
                self.bounded_methods(&body.stmts, &self_ty, bounds);
                self.leave_body(scope);
            }
            StmtKind::Use { .. } | StmtKind::Break(_) | StmtKind::Continue(_) | StmtKind::Error => {
            }
            StmtKind::Expr(expr) => return self.infer(expr),
            StmtKind::Assign { target, op, value } => {
                // Check `@cache` field written from outside its struct (`L0609`).
                {
                    let mut root = target;
                    while let ExprKind::Index { base: inner, .. } = &root.kind {
                        root = inner;
                    }
                    if let ExprKind::Field {
                        base: obj,
                        name: field,
                    } = &root.kind
                    {
                        let obj_ty = self.infer(obj);
                        if let Ty::Named { name: sname, .. } = obj_ty {
                            let inside_same = matches!(
                                self.self_ty.as_ref(),
                                Some(Ty::Named { name, .. }) if name == &sname
                            );
                            if !inside_same {
                                let is_cached = matches!(
                                    self.type_def(&sname).as_deref(),
                                    Some(Def::Struct { cached_fields, .. })
                                        if cached_fields.iter().any(|c| c == &field.text)
                                );
                                if is_cached {
                                    self.sink.push(cpl_diag::msgs::check::CACHE_FIELD_WRITTEN.warning(
                                        target.span,
                                        &[&sname, &field.text, &sname],
                                    ).with_help(format!(
                                        "`{sname}`'s methods keep a `@cache` field in step with the fields it derives from; \
                                         write it from one of them, or drop `@cache` if it is plain state"
                                    )));
                                }
                            }
                        }
                    }
                }
                let prev_assigning = self.assigning_name;
                if *op == AssignOp::Set && matches!(target.kind, ExprKind::Name(_)) {
                    self.assigning_name = Some(target.span);
                }
                let target_ty = self.assign_target(target);
                let prev_bounds_range = self.expected_bounds_range;
                self.expected_bounds_range |= Self::expects_bounds_range(&target_ty);
                self.assigning_name = prev_assigning;
                let value_ty = match self.part_map(target, *op, value) {
                    Some(ty) => ty,
                    None => self.infer(value),
                };
                self.expected_bounds_range = prev_bounds_range;
                if *op == AssignOp::Set {
                    self.note_read(value, &target_ty);
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
                let before_mutable = self.sink.len();
                self.require_mutable(target, "assigned");
                let not_assignable = self.sink.len() > before_mutable;
                if let ExprKind::Field { base, name } = &target.kind {
                    let base_ty = self.quietly(|c| c.infer(base));
                    if let Ty::Named { name: sname, .. } = &base_ty {
                        let fixed_field = match self.type_def(sname).as_deref() {
                            Some(Def::Struct {
                                fields,
                                var_fields,
                                cached_fields,
                                ..
                            }) => {
                                fields.iter().any(|f| f.name == name.text)
                                    && !var_fields.iter().any(|f| f == &name.text)
                                    && !cached_fields.iter().any(|f| f == &name.text)
                            }
                            _ => false,
                        };
                        if fixed_field {
                            let var_at = self.env.field_decl(sname, &name.text);
                            let base_text = self.quote(base.span);
                            if let Some(diag) = self.report(
                                target.span,
                                msgs::check::ASSIGN_FIXED_FIELD_FIELD_IS_FIXED_NOT,
                                &[&sname, &name.text],
                            ) {
                                diag.help = Some(match var_at {
                                    Some(_) => format!(
                                        "declare `var {}` on `{sname}` or rebuild with `{base_text} with {}: ...`",
                                        name.text, name.text
                                    ),
                                    None => format!(
                                        "`var {}` goes on its own line in a block `struct {sname}:`; or rebuild with `{base_text} with {}: ...`",
                                        name.text, name.text
                                    ),
                                });
                                if let Some(at) = var_at {
                                    diag.fix(format!("declare `var {}`", name.text), at, "var ");
                                }
                            }
                        }
                    }
                }

                self.check_access(target, true);
                if *op == AssignOp::Set && !not_assignable {
                    self.range_store(value, &target_ty);
                }
                if let Some((root, _)) = Self::root_name(target) {
                    self.check_memo_capture_write(root, target.span);
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
                    self.escape_unless_kept(value, "this assignment", Keep::In(target));
                    if let Some((root, _)) = Self::root_name(target) {
                        let target_depth = self.var_depth(root);
                        self.no_escape_assign(value, target_depth);
                    }
                }
                if matches!(op, AssignOp::Max | AssignOp::Min) {
                    self.row_keep(target, &target_ty, value, &value_ty);
                }
                let row = |ty: &Ty| {
                    matches!(ty, Ty::Array { .. })
                        || matches!(ty, Ty::Named { name, .. } if name == "List")
                };
                let target_ty =
                    if *op == AssignOp::Set && part && row(&target_ty) && !row(&value_ty) {
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
                        self.report(
                            value.span,
                            msgs::check::TYPE_MISMATCH_IS_A_ROW_OF_2,
                            &[
                                &self.quote(target.span),
                                &want,
                                &self.quote(value.span),
                                &value_ty.with_article(),
                            ],
                        );
                    }
                } else if *op == AssignOp::Set && !not_assignable {
                    self.expect_moves(&value_ty, &target_ty, value, |c, got, want| {
                        cpl_diag::msgs::check::BUT.at(&[
                            &c.quote(target.span),
                            &want.with_article(),
                            &c.quote(value.span),
                            &got.with_article(),
                        ])
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
                self.begin_cond_hold(cond, body);
                let before = self.moved.clone();
                self.block(body);
                self.lend_reset(mark);
                self.pop_scope();
                self.no_move_across_rounds(&before, body);
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
                if let ExprKind::Field {
                    base: obj,
                    name: field,
                } = &iter.kind
                {
                    if matches!(&obj.kind, ExprKind::SelfValue)
                        && self.cached_fields.iter().any(|c| c == &field.text)
                    {
                        self.check_cache_lend_body(body, &field.text);
                    }
                }
                self.loop_advice(pat, iter, stmt.span);
                if matches!(
                    iter.kind,
                    ExprKind::Range {
                        lo: None,
                        hi: None,
                        ..
                    }
                ) {
                    if let Some(diag) = self.report(
                        iter.span,
                        cpl_diag::msgs::check::BARE_MEANS_ALL_DIMENSION,
                        &[],
                    ) {
                        diag.help =
                            Some("name what to walk: `for x in D` or `for i in 0..n`".into());
                    }
                }
                let prev_iterating = std::mem::replace(
                    &mut self.iterating_range,
                    matches!(iter.kind, ExprKind::Range { .. }),
                );
                let iter_ty = self.infer(iter);
                self.iterating_range = prev_iterating;
                let elem = self.walk_elem_ty(iter, &iter_ty);
                let steerable = matches!(&iter_ty, Ty::Named { name, .. } if name == "Gen")
                    && matches!(&elem, Ty::Named { .. });
                self.push_scope();
                self.drop_facts_written(body, Some(pat));
                self.bind(pat, &elem, steerable, "for");
                self.note_pieces(pat, iter);
                let mark = self.lend_mark();
                self.hold_iterated(stmt, iter, &iter_ty);
                let before = self.moved.clone();
                self.block(body);
                self.lend_reset(mark);
                self.pop_scope();
                self.no_move_across_rounds(&before, body);
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
                        let before = &self.src[..value.span.start as usize];
                        let at = before
                            .strip_suffix(' ')
                            .filter(|b| b.ends_with("return"))
                            .map(|b| Span::new(b.len() as u32, value.span.end));
                        let diag = self.report(
                            value.span,
                            msgs::check::UNEXPECTED_TOKEN_A_TOP_LEVEL_RETURN,
                            &[],
                        );
                        if let Some(diag) = diag {
                            diag.help = Some(
                                "write a bare `return`: it ends the program, or the test case below `@!multitest`"
                                    .to_string(),
                            );
                            if let Some(at) = at {
                                diag.fix("write a bare `return`", at, "");
                            }
                        }
                    }
                }
                if let (Some(value), Some(value_ty)) = (value, value_ty) {
                    if !(self.fns.is_empty() && matches!(value_ty, Ty::Unit)) {
                        self.returns(value, &value_ty);
                        self.undeclared_return(value, &value_ty);
                        if !self.fn_lends.last().copied().unwrap_or(false) {
                            self.no_escape(value, "`return`");
                        }
                        self.no_escape_return(value, "`return`");
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
            StmtKind::Context { name, fields } => {
                let mut fs = Vec::new();
                for f in fields {
                    let fty = self.field_type(f);
                    if let Some(d) = &f.default {
                        let dty = self.infer(d);
                        self.expect_fits(&dty, &fty, d, |c, got, want| {
                            cpl_diag::msgs::check::BUT.at(&[
                                &f.name.text,
                                &want.with_article(),
                                &c.quote(d.span),
                                &got.with_article(),
                            ])
                        });
                    }
                    fs.push((f.name.text.clone(), fty));
                }
                self.declare(&name.text, name.span, Ty::Record(fs), false, "context");
            }
            StmtKind::With { ctx, args, body } => {
                self.check_with(ctx, args);
                if let Some(body) = body {
                    return self.block(body);
                }
            }
            StmtKind::Law { params, body, .. } => {
                self.push_scope();
                if let Some(self_ty) = self.self_ty.clone() {
                    self.declare("self", stmt.span, self_ty, false, "self");
                }
                for param in params {
                    let ty = match &param.ty {
                        Some(ty) => self.resolve_type(ty),
                        None => Ty::Unknown,
                    };
                    self.declare(&param.name.text, param.name.span, ty, false, "param");
                }
                let ty = self.infer(body);
                self.expect_fits(&ty, &Ty::Bool, body, |c, got, _| {
                    cpl_diag::msgs::check::LAW_MUST_RETURN_BOOL
                        .at(&[&c.quote(body.span), &got.with_article()])
                });

                self.pop_scope();
            }
            StmtKind::Lend { place, .. } => {
                self.infer(place);
                self.lend_in_place(stmt);
            }
            StmtKind::Mod { .. } => {}
        }
        Ty::Unit
    }

    /// Checks a returned value against the enclosing fn's declared type.
    /// A fn's value `read()`, or `ask(..)` of fixed-size arguments, reads a
    /// value of its declared return type.
    fn fixed_read(&mut self, value: &Expr, ret: &Ty) {
        if let ExprKind::Call { callee, args } = &value.kind {
            match &callee.kind {
                ExprKind::Name(n) if n == "read" && args.is_empty() && self.io_fixed(ret) => {
                    self.fixed_io.push(callee.span)
                }
                ExprKind::Name(n) if n == "ask" && self.io_fixed(ret) => {
                    self.record_io(callee.span, n, args);
                }
                _ => {}
            }
        }
    }

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
        allowed.extend(self.packed_domains(value_ty));
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
            cpl_diag::msgs::check::RETURNS_BUT.at(&[
                &name,
                &want.with_article(),
                &c.quote(value.span),
                &got.with_article(),
            ])
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
}

impl Checker<'_> {}

/// The span of a `break` or `continue` that leaves this loop, outside any
/// loop nested in it.
fn loop_exit(stmts: &[Stmt]) -> Option<Span> {
    stmts.iter().find_map(|s| match &s.kind {
        StmtKind::Break(None) | StmtKind::Continue(None) => Some(s.span),
        StmtKind::Expr(Expr {
            kind: ExprKind::If {
                branches,
                otherwise,
            },
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
        ast::PatternKind::Tuple(_) | ast::PatternKind::Record { .. } => pat.parts().any(refutable),
        ast::PatternKind::Or(alts) => alts.iter().all(refutable),
        _ => false,
    }
}

/// A statement that does work when reached, unlike a declaration; a bare
/// `with` line may not follow one. (R09.1.1)
pub(crate) fn runs(stmt: &Stmt) -> bool {
    matches!(
        stmt.kind,
        StmtKind::Let { .. }
            | StmtKind::Expr(..)
            | StmtKind::Assign { .. }
            | StmtKind::While { .. }
            | StmtKind::For { .. }
            | StmtKind::Return(..)
            | StmtKind::Break(..)
            | StmtKind::Continue(..)
            | StmtKind::Yield { .. }
            | StmtKind::Assert { .. }
            | StmtKind::Unchecked(..)
            | StmtKind::With { body: Some(_), .. }
    )
}

impl<'a> Checker<'a> {}

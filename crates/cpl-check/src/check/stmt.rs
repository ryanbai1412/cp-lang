//! Statements, blocks, and declarations.

use super::pattern::{name_spans, pattern_types};
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
    fn dropped_result(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, .. } = &expr.kind else {
            return;
        };
        let ExprKind::Name(name) = &callee.kind else {
            return;
        };
        if self.local(name).is_some() || !self.env.names().contains(name) {
            return;
        }
        let Some(def) = self.global(name) else {
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
        let message = format!(
            "`{shown}` returns {}, and it is dropped",
            ret.with_article()
        );
        self.sink.push(
            Diagnostic::warning(codes::UNUSED_VALUE, expr.span, message).with_help(format!(
                "use it (`let r = {shown}`), or mark `{name}` `@droppable` if its result is optional"
            )),
        );
    }

    /// R07.9.2: `s.fold(f)` with a plain fn has no identity, so it is `?T`.
    fn fold_needs_identity(&mut self, init: &Expr, declared: &Ty) {
        if matches!(declared, Ty::Optional(_) | Ty::Error) {
            return;
        }
        let ExprKind::Call { callee, args } = &crate::check::lends::unparen(init).kind else {
            return;
        };
        let ExprKind::Field { name, .. } = &callee.kind else {
            return;
        };
        let [arg] = args.as_slice() else {
            return;
        };
        if name.text != "fold"
            || arg.name.is_some()
            || !matches!(arg.value.kind, ExprKind::Lambda { .. })
        {
            return;
        }
        let shown = self.quote(init.span);
        let message = format!(
            "`{shown}` is {}: a fold by a lambda has no identity, so an empty sequence gives `None`",
            Ty::Optional(Box::new(declared.clone())).with_article()
        );
        if let Some(diag) = self.error(codes::EMPTY_FOLD_NO_IDENTITY, init.span, message) {
            diag.help = Some(format!("unwrap: `{shown}!`, or `?? d` for a default"));
            diag.fix("unwrap it with `!`", Span::point(init.span.end), "!");
        }
    }

    /// R08.5.2: inherent methods live with their type.
    fn impl_home(
        &mut self,
        trait_name: Option<&cpl_syntax::ast::Type>,
        target: &cpl_syntax::ast::Type,
    ) {
        let shown = self.quote(target.span);
        match trait_name {
            None if shown.contains('.') => {
                let message =
                    format!("`impl {shown}:` adds inherent methods outside `{shown}`'s module");
                if let Some(diag) = self.error(codes::UNSUPPORTED, target.span, message) {
                    diag.help = Some(format!(
                        "move the methods into `{shown}`'s module, or declare a trait and `impl` it"
                    ));
                }
            }
            _ => {}
        }
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
            format!("`{shown}` is outside the range type `{text}`")
        } else {
            format!("`{shown}` is {is}, outside the range type `{text}`")
        };
        let text = text.clone();
        if let Some(diag) = self.error(codes::RANGE_STORE_CONST, value.span, message) {
            diag.help = Some(format!(
                "store a value in `{text}`, or widen the declared type"
            ));
        }
    }

    /// 0.3 writes `enum Dir = N | E`; `enum Dir: N, E` is gone.
    fn no_comma_variants(&mut self, variants: &[cpl_syntax::ast::Variant]) {
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
            let message = "an enum's variants are separated by `|`, not `,`".to_string();
            if let Some(diag) = self.error(codes::EXPECTED_TOKEN, a.span.to(b.span), message) {
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
                    format!("`{keyword} {name} = read()` doesn't say what to read"),
                    format!("annotate it: `{keyword} {name}: Int = read()`"),
                )
            }
            _ => return false,
        };
        if let Some(diag) = self.error(codes::INFERENCE_NEEDS_ANNOTATION, init.span, message) {
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
            format!("`{keyword} {name}`: values are lowercase; `{name}` would read as a type");
        let fix = self.const_fix(keyword, pat.span, init);
        if let Some(diag) = self.error(codes::VALUE_NAME_CASE, pat.span, message) {
            diag.help = Some(format!("rename it `{lower}`, or write `const {name}`"));
            diag.fixes.extend(fix);
        }
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
                StmtKind::With { body: None, .. } if !leading => {
                    let message = "a bare `with` line comes first in its block; indent what it covers under `with ...:`".to_string();
                    self.error(codes::WITH_NOT_FIRST, stmt.span, message);
                }
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
            self.end_holds(stmt);
        }
        self.present.truncate(present);
        self.memo_captured.truncate(memo_captured);
        self.local_fn_captures.truncate(local_fn_captures);
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
                if self.some_pattern(path) {
                    let inner = match ty {
                        Ty::Optional(inner) => (**inner).clone(),
                        _ => Ty::Unknown,
                    };
                    if let Some(arg) = args.first() {
                        self.bind(arg, &inner, mutable, keyword);
                    }
                    return;
                }
                let fields = match path.last().and_then(|n| self.global(&n.text)) {
                    Some(def) => def.payload_at(ty),
                    None => Vec::new(),
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
                indices.len() > 1
                    && indices
                        .iter()
                        .any(|i| matches!(i.kind, ExprKind::Range { .. }))
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
        let ExprKind::Binary {
            op: bin, lhs, rhs, ..
        } = &value.kind
        else {
            return None;
        };
        let worked = matches!(
            bin,
            ast::BinOp::Add
                | ast::BinOp::Sub
                | ast::BinOp::Mul
                | ast::BinOp::BitAnd
                | ast::BinOp::BitOr
                | ast::BinOp::BitXor
        );
        if op != AssignOp::Set || !worked || !is_part(target) || is_part(lhs) == is_part(rhs) {
            return None;
        }
        let (part, one) = if is_part(lhs) { (lhs, rhs) } else { (rhs, lhs) };
        let (part_ty, one_ty) = (self.infer(part), self.infer(one));
        let cell = self.element_ty(&part_ty);
        let row = |ty: &Ty| {
            matches!(ty, Ty::Array { .. }) || matches!(ty, Ty::Named { name, .. } if name == "List")
        };
        if part_ty == Ty::Error || one_ty == Ty::Error {
            return Some(Ty::Error);
        }
        let number = cell.is_int() || cell == Ty::Float;
        if !row(&part_ty)
            || row(&one_ty)
            || !number
            || !(one_ty.is_vague() || self.fits(&one_ty, &cell))
        {
            let message = format!(
                "`{}` works each cell of `{}` with one number, but `{}` is {}",
                self.quote(value.span),
                self.quote(part.span),
                self.quote(one.span),
                one_ty.with_article()
            );
            if let Some(diag) = self.error(codes::BAD_OPERANDS, value.span, message) {
                diag.help = Some(
                    "the cells and the value are one kind of number: `a[l..r] = b[l2..r2] + x`"
                        .to_string(),
                );
            }
            return Some(Ty::Error);
        }
        Some(part_ty)
    }

    /// `a max= b` on a row (an array of one dimension, or a list) goes
    /// cell by cell, so `b` is a row of the same type.
    pub(crate) fn row_keep(
        &mut self,
        target: &ast::Expr,
        target_ty: &Ty,
        value: &ast::Expr,
        value_ty: &Ty,
    ) {
        // `a[l..r] max= x`, `g[r, ..] min= x`: one value against every cell
        // of the part.
        let part = matches!(&target.kind, ExprKind::Index { indices, .. } if indices.iter().any(|i| matches!(i.kind, ExprKind::Range { .. })));
        let cell = match target_ty {
            Ty::Array { elem, .. } => Some(&**elem),
            Ty::Named { name, args } if name == "List" && args.len() == 1 => Some(&args[0]),
            _ => None,
        };
        if let (true, Some(cell)) = (part, cell) {
            let row = matches!(value_ty, Ty::Array { .. })
                || matches!(value_ty, Ty::Named { name, .. } if name == "List");
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
    /// `In + Out` in a `domain` declaration: the disjoint union of domains.
    fn domain_sum(&mut self, def: &ast::Expr) -> bool {
        let ExprKind::Binary {
            op: ast::BinOp::Add,
            lhs,
            rhs,
            ..
        } = &def.kind
        else {
            return false;
        };
        let part = |c: &mut Self, e: &ast::Expr| {
            c.domain_sum(e)
                || matches!(e.kind, ExprKind::Name(_)) && matches!(c.infer(e), Ty::Domain(_))
        };
        part(self, lhs) && part(self, rhs)
    }

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
        if top_level {
            if let StmtKind::Expr(expr) = &stmt.kind {
                self.dropped_result(expr);
            }
        }
        if let StmtKind::Expr(expr) = &stmt.kind {
            if let ExprKind::Try(inner) = &expr.kind {
                let message = format!(
                    "the result of `{}` is dropped: `?` makes its expression `None`, it never returns early",
                    self.quote(expr.span)
                );
                let help = format!(
                    "to leave on `None`, bind it: `let v = {} else: return None` (R02.4.7)",
                    self.quote(inner.span)
                );
                if let Some(diag) = self.error(codes::DROPPED_QUESTION, expr.span, message) {
                    diag.help = Some(help);
                }
            }
        }
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
                if let (Some(ast_ty), Some(init_expr)) = (ty, init) {
                    if let ast::TypeKind::ConstrainedStr(alphabet) = &ast_ty.kind {
                        if let ExprKind::Str(pieces) = &init_expr.kind {
                            for piece in pieces {
                                if let cpl_syntax::ast::StrPiece::Text(text) = piece {
                                    if let Some(bad) = text.chars().find(|c| !alphabet.contains(*c))
                                    {
                                        self.error(
                                            codes::RANGE_STORE_CONST,
                                            init_expr.span,
                                            format!("`{bad}` is not in `Str of \"{alphabet}\"`"),
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
                let unannotated =
                    declared.is_none() && self.needs_annotation(keyword, pat, init.as_ref());
                let mut init_ty = init.as_ref().map(|init| self.infer(init));
                if unannotated {
                    init_ty = Some(Ty::Error);
                }
                if let (PatternKind::Name(name), Some(init)) = (&pat.kind, init.as_ref()) {
                    if let Some(rebased) = self.rebased(name, init) {
                        init_ty = Some(rebased);
                    }
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
                }
                // `var seen: [V]Bool = false` fills the array with the value.
                // A comprehension builds the array when one is declared.
                let fills = match (&declared, &init_ty, init) {
                    (Some(decl), Some(ity), Some(init_expr)) => {
                        self.array_fills(decl, ity, init_expr)
                    }
                    _ => false,
                };
                let view = match (&declared, init) {
                    (Some(Ty::Named { name, .. }), Some(init)) if name == "List" => {
                        self.view_kind(init)
                    }
                    _ => None,
                };
                if let (Some(declared), Some(init), Some(kind)) = (&declared, init, view) {
                    let kind = if kind == "slice" { "slice view" } else { kind };
                    let init_text = self.quote(init.span);
                    let message = format!(
                        "`{}` is declared {}, but `{init_text}` is a {kind}, not a `List`",
                        self.quote(pat.span),
                        declared.with_article(),
                    );
                    if let Some(diag) = self.error(codes::TYPE_MISMATCH, init.span, message) {
                        diag.help = Some(format!(
                            "a view borrows its source; copy it with `List({init_text})` (R07.2.11)"
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
                    self.expect_moves(init_ty, declared, init, |c, got, want| {
                        format!(
                            "`{name}` is declared {}, but `{}` is {}",
                            want.with_article(),
                            c.quote(init.span),
                            got.with_article()
                        )
                    });
                    self.empty_braces(declared, init);
                }
                let guessed = declared.is_none()
                    && self.sink.encloses_poison(stmt.span)
                    && crate::db::head_leaves_open(self.src, stmt);
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
                        let message = format!(
                            "`{name}` needs a type annotation because `None` has no element type"
                        );
                        if let Some(diag) =
                            self.error(codes::INFERENCE_NEEDS_ANNOTATION, pat.span, message)
                        {
                            diag.help = Some(format!("write `{keyword} {name}: ?Int = None`"));
                        }
                    } else if is_bare_read {
                        let name = self.quote(pat.span);
                        let message =
                            "`read()` needs a type annotation to know what to read".to_string();
                        if let Some(diag) =
                            self.error(codes::INFERENCE_NEEDS_ANNOTATION, pat.span, message)
                        {
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
                        let message = format!(
                            "value `{name}` starts with an uppercase letter; values are lowercase (`{name}` reads as a type)"
                        );
                        self.error(codes::VALUE_NAME_CASE, pat.span, message);

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
                let prev_cached = std::mem::replace(
                    &mut self.cached_fields,
                    s.fields
                        .iter()
                        .filter(|f| f.cached)
                        .map(|f| f.name.text.clone())
                        .collect(),
                );
                self.methods(&s.methods, &self_ty);
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
                self.no_comma_variants(&e.variants);
                let mut seen_tags: Vec<(String, Span)> = Vec::new();
                for variant in &e.variants {
                    if let Some(tag_expr) = &variant.tag {
                        let tag_text = self.quote(tag_expr.span);
                        if let Some((_, prev_span)) = seen_tags.iter().find(|(t, _)| t == &tag_text)
                        {
                            let message = format!(
                                "enum `{}` already has a variant with `@tag({tag_text})`",
                                e.name.text
                            );
                            if let Some(diag) =
                                self.error(codes::DUPLICATE_VARIANT_TAG, tag_expr.span, message)
                            {
                                super::advice::duplicate_tag(diag, *prev_span);
                            }
                        } else {
                            seen_tags.push((tag_text, tag_expr.span));
                        }
                    }
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
                self.methods(&body.stmts, &self_ty);
                self.param_bounds.truncate(outer_bounds);
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
                self.impl_home(trait_name.as_ref(), target);
                self.bounds_exist(bounds);
                if trait_name.is_none() && self.quote(target.span).contains('.') {
                    self.error(
                        codes::UNSUPPORTED,
                        target.span,
                        "inherent `impl` must be in the same module as the type".to_string(),
                    );
                } else if let Some(tr) = trait_name {
                    let file_names = self.env.declared();
                    let target_head = type_head(target);
                    let trait_head = type_head(tr);
                    let target_local = target_head.as_ref().is_some_and(|t| file_names.contains(t));
                    let trait_local = trait_head.as_ref().is_some_and(|t| file_names.contains(t));
                    if !target_local
                        && !trait_local
                        && !matches!(&self_ty, Ty::Param(_) | Ty::Unknown | Ty::Error)
                    {
                        self.error(
                            codes::UNSUPPORTED,
                            tr.span,
                            format!(
                                "orphan `impl`: neither `{}` nor `{}` is defined in this module",
                                self.quote(tr.span),
                                self.quote(target.span)
                            ),
                        );
                    }
                }

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
                                    self.sink.push(Diagnostic::warning(
                                        codes::CACHE_FIELD_WRITTEN_OUTSIDE,
                                        target.span,
                                        format!(
                                            "`{}.{}` is a `@cache` field and should only be written inside `{sname}`",
                                            sname, field.text
                                        ),
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
                            let message = format!(
                                "field `{sname}.{}` is fixed (not `var`) and cannot be assigned",
                                name.text
                            );
                            if let Some(diag) =
                                self.error(codes::ASSIGN_FIXED_FIELD, target.span, message)
                            {
                                diag.help = Some(format!(
                                    "declare `var {}` on `{sname}` or rebuild with `{base_text} with {}: ...`",
                                    name.text, name.text
                                ));
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
                    self.no_escape(value, "this assignment");
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
                        let message = format!(
                            "`{}` is a row of {want}, but `{}` is {}",
                            self.quote(target.span),
                            self.quote(value.span),
                            value_ty.with_article()
                        );
                        self.error(codes::TYPE_MISMATCH, value.span, message);
                    }
                } else if *op == AssignOp::Set && !not_assignable {
                    self.expect_moves(&value_ty, &target_ty, value, |c, got, want| {
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
                self.counter_over_slice(iter, stmt.span);
                if matches!(
                    iter.kind,
                    ExprKind::Range {
                        lo: None,
                        hi: None,
                        ..
                    }
                ) {
                    if let Some(diag) = self.error(
                        codes::EXPECTED_EXPR,
                        iter.span,
                        "a bare `..` means all of a dimension, so it only goes inside `[...]`"
                            .to_string(),
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
                let elem = self.element_ty(&iter_ty);
                let steerable = matches!(&iter_ty, Ty::Named { name, .. } if name == "Gen")
                    && matches!(&elem, Ty::Named { .. });
                self.push_scope();
                self.drop_facts_written(body, Some(pat));
                self.bind(pat, &elem, steerable, "for");
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
                            format!(
                                "`{}` is {}, but `{}` is {}",
                                f.name.text,
                                want.with_article(),
                                c.quote(d.span),
                                got.with_article()
                            )
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
                    format!(
                        "a `law` must return a `Bool`, but `{}` is {}",
                        c.quote(body.span),
                        got.with_article()
                    )
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
                    if !self.domain_sum(def) {
                        self.infer(def);
                    }
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

    /// The `K` of an `impl Index`: how many values the type has. `const K` is known at compile
    /// time; `let K` is evaluated once, where the impl is declared, and frozen, so it can't read
    /// a `var` (plan/19 §3.1).
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
                } if super::pattern::domain_binding(pat) == Some("K") => {
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
                if matches!(trait_ty.kind, ast::TypeKind::Row(_)) {
                    continue;
                }
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
                    in_lambda: false,
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
        if let Some(Def::Trait {
            supers,
            alias: true,
            ..
        }) = type_head(trait_ty).and_then(|n| self.global(&n)).as_deref()
        {
            let impls = supers
                .iter()
                .map(|s| format!("`impl {s} for {}`", self.quote(target.span)))
                .collect::<Vec<_>>()
                .join(" and ");
            let message = format!(
                "`{trait_text}` is a trait alias: it names bounds and has no impl of its own"
            );
            if let Some(diag) = self.error(codes::UNSUPPORTED, trait_ty.span, message) {
                diag.help = Some(format!("write {impls} instead"));
            }
            return;
        }
        // Check supertrait requirements (R08.1.3): `impl Sub for T` requires `impl Super for T`.
        if let ast::TypeKind::Named { name, .. } = &trait_ty.kind {
            if let Some(Def::Trait { supers, .. }) = self.global(&name.text).as_deref().cloned() {
                let target_ty = self.resolve_type(target);
                for sup in supers {
                    if !self.implements_trait(&target_ty, &sup) {
                        let target_text = self.quote(target.span);
                        let message = format!(
                            "`impl {trait_text} for {target_text}` requires `impl {sup} for {target_text}`"
                        );
                        self.error(codes::MISSING_IMPL, trait_ty.span, message);
                    }
                }
            }
        }
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
                        Some(s) => diag.suggest(name.span, s),
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

    /// Reports a `@memo` that would not remember anything. Both back ends keep one table of
    /// answers per top-level or local fn, keyed by its arguments, and run the fn as written
    /// when they can't: the program would be right and, silently, as slow as without `@memo`.
    fn is_bounded_memo_type(&mut self, ty: &Ty) -> bool {
        match ty {
            Ty::Pos(_) | Ty::Bool | Ty::Range { .. } | Ty::Error | Ty::Unknown => true,
            Ty::Named { name, .. } => {
                name == "Cut" || matches!(self.type_def(name).as_deref(), Some(Def::Enum { .. }))
            }
            Ty::Tuple(ts) => ts.iter().all(|t| self.is_bounded_memo_type(t)),
            Ty::Optional(inner) => self.is_bounded_memo_type(inner),
            _ => false,
        }
    }

    /// Outer `var`s referenced inside a `@memo` fn body.
    fn memo_captured_vars(&self, f: &ast::FnDecl) -> Vec<String> {
        let mut param_names = std::collections::HashSet::new();
        for p in &f.params {
            param_names.insert(p.name.text.clone());
        }
        let mut names = Vec::new();
        match &f.body {
            FnBody::Expr(e) => names.extend(super::lends::names_in(e)),
            FnBody::Block(b) => {
                for s in &b.stmts {
                    let (ns, _) = super::lends::item_names(std::slice::from_ref(s));
                    names.extend(ns);
                }
            }
            FnBody::None => {}
        }
        let mut out = Vec::new();
        for n in names {
            if param_names.contains(&n) || out.contains(&n) {
                continue;
            }
            if self.local(&n).is_some_and(|l| l.mutable) {
                out.push(n);
            }
        }
        out
    }

    pub(crate) fn check_memo_capture_write(&mut self, root: &str, span: Span) {
        if let Some((_, fn_name, memo_span)) = self
            .memo_captured
            .iter()
            .rev()
            .find(|(v, _, _)| v == root)
            .cloned()
        {
            let message =
                format!("`{root}` can't be modified while `@memo` fn `{fn_name}` captures it");
            if let Some(diag) = self.error(codes::BAD_MEMO, span, message) {
                diag.labels.push(cpl_diag::Label {
                    span: memo_span,
                    message: format!("`{fn_name}` is memoized here and reads `{root}`"),
                });
                diag.help = Some(format!(
                    "pass `{root}` as a parameter to `{fn_name}`, or finish updating `{root}` before declaring `{fn_name}`"
                ));
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn memo_fits(
        &mut self,
        f: &ast::FnDecl,
        params: &[crate::Param],
        self_ty: Option<&Ty>,
        generator: bool,
        unit: bool,
        memo: Span,
        is_array: bool,
    ) {
        let name = &f.name.text;
        let changed = f.params.iter().find(|p| p.mutable && p.name.text != "self");
        if is_array {
            for (ast_p, ty_p) in f.params.iter().zip(params.iter()) {
                if !self.is_bounded_memo_type(&ty_p.ty) {
                    let pname = &ast_p.name.text;
                    let pty = &ty_p.ty;
                    let message = format!(
                        "`@memo(array)` on `{name}` requires bounded parameter types, but `{pname}: {pty}` is unbounded"
                    );
                    if let Some(diag) = self.error(codes::BAD_MEMO, ast_p.name.span, message) {
                        diag.help = Some(
                            "give the parameter a bounded type (a domain, range, `Bool`, or enum), or use `@memo(map)`".into(),
                        );
                    }
                    return;
                }
            }
        }
        let (message, help) = if let Some(owner) = self_ty {
            let owner = owner.to_string();
            (
                format!("`@memo` can't remember `{name}`: it is a fn of `{owner}`"),
                format!(
                    "only a top-level or local fn keeps a table of its answers: move `{name}` out of `{owner}` and pass it what it reads"
                ),
            )
        } else if let Some(param) = changed {
            let param = &param.name.text;
            (
                format!("`@memo` can't remember `{name}`: it changes its argument `{param}`"),
                format!(
                    "a remembered answer is given back without running the fn, so `{param}` would be left as it was: take `{param}` without `mut`, or drop `@memo`"
                ),
            )
        } else if generator {
            (
                format!("`@memo` can't remember `{name}`: it yields its values one by one"),
                "collect the values into a list and return that: `-> List<T>`".to_string(),
            )
        } else if !f.generics.is_empty() {
            (
                format!("`@memo` can't remember the generic fn `{name}`"),
                format!(
                    "there is one table of answers per fn, and `{name}` is a different fn for each type it is called with: write it for the type it is used with"
                ),
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

    /// `memo` is where the fn's `@memo` is written, when it has one, and whether it is `@memo(array)`.
    fn fn_decl(
        &mut self,
        f: &ast::FnDecl,
        self_ty: Option<&Ty>,
        local: bool,
        memo: Option<(Span, bool)>,
    ) {
        if let Some(lends) = &f.lends {
            let mut returned = Vec::new();
            match &f.body {
                ast::FnBody::Expr(e) => returned.push(e),
                ast::FnBody::Block(b) => Self::collect_returned_exprs(b, &mut returned),
                ast::FnBody::None => {}
            }
            if !lends.sources.is_empty() {
                for e in &returned {
                    let root = Self::expr_root_name(e);
                    let ok = root.is_some_and(|r| lends.sources.iter().any(|s| s.text == r));
                    if !ok {
                        self.error(
                            codes::LENT_ESCAPES,
                            e.span,
                            format!(
                                "lent value must come from `{}`",
                                lends
                                    .sources
                                    .iter()
                                    .map(|s| s.text.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        );
                    }
                }
            }
            if let Some(Ty::Named { name: sname, .. }) = self_ty {
                let cached = match self.type_def(sname).as_deref() {
                    Some(Def::Struct { cached_fields, .. }) => cached_fields.clone(),
                    _ => Vec::new(),
                };
                if !cached.is_empty() {
                    for e in &returned {
                        if let Some(cf) = Self::expr_self_field(e) {
                            if cached.iter().any(|c| c == cf) {
                                self.error(
                                    codes::BAD_PROJECTION,
                                    e.span,
                                    format!(
                                        "cannot lend `@cache` field `self.{cf}` from `{sname}`"
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }
        let saved_row_bounds = self.row_bounds.clone();
        let outer_for_row = self.generics.len();
        self.generics
            .extend(f.generics.iter().map(|g| g.text.clone()));
        for b in &f.bounds {
            for t in &b.traits {
                match &t.kind {
                    ast::TypeKind::Row(fields) => {
                        let resolved = fields
                            .iter()
                            .map(|(n, ty)| (n.text.clone(), self.resolve_type(ty)))
                            .collect();
                        self.row_bounds.insert(b.name.text.clone(), resolved);
                    }
                    ast::TypeKind::Named { name: tname, .. } => {
                        if let Some(Def::Trait { fields, .. }) =
                            self.global(&tname.text).as_deref().cloned()
                        {
                            if !fields.is_empty() {
                                self.row_bounds
                                    .entry(b.name.text.clone())
                                    .or_default()
                                    .extend(fields);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        self.generics.truncate(outer_for_row);
        let sig = self.fn_sig(f, self_ty);
        let Ty::Fn { params, ret } = &sig else {
            self.row_bounds = saved_row_bounds;
            return;
        };
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
        for ctx in &f.uses {
            if !matches!(ctx.text.as_str(), "checks" | "lint")
                && self.context_record_fields(&ctx.text).is_none()
            {
                self.error(
                    codes::UNRESOLVED_NAME,
                    ctx.span,
                    format!("unknown context `{}` in `uses` clause", ctx.text),
                );
            }
        }
        let outer = self.generics.len();
        let outer_bounds = self.param_bounds.len();
        self.generics
            .extend(f.generics.iter().map(|g| g.text.clone()));
        self.param_bounds
            .extend(super::types::extract_bounds(&f.bounds));
        self.bounds_exist(&f.bounds);
        self.in_fn += 1;
        // A move is a fact about one body: a later fn may name a field or global again.
        let outer_moved = std::mem::take(&mut self.moved);
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
            if let Some(at) = lends.bare {
                if let Some(diag) = self.error(
                    codes::EXPECTED_TOKEN,
                    at,
                    "expected `->` before `lends`: a lend is the fn's result".to_string(),
                ) {
                    diag.help = Some("write `-> lends T`".to_string());
                    diag.fixes.push(cpl_diag::Fix {
                        title: "insert `->`".to_string(),
                        edits: vec![(Span::new(at.start, at.start), "-> ".to_string())],
                    });
                }
            }
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
            if let Some(ast::Type {
                kind: ast::TypeKind::LiteralUnion(lits),
                ..
            }) = &param.ty
            {
                self.lit_unions
                    .insert(param.name.text.clone(), lits.clone());
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
        if self.scopes.len() > 1 {
            self.local_effects
                .insert(f.name.text.clone(), super::lends::own_effects(f));
        }
        self.note_ret_slot(f);
        let fn_depth = self.scopes.len().saturating_sub(1);
        self.fns.push((
            f.name.text.clone(),
            (**ret).clone(),
            f.ret.is_some() && !generator,
            fn_depth,
        ));
        self.fn_lends.push(f.lends.is_some());
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
                            c.fixed_read(last, ret);
                            c.returns(last, &ty);
                            if !c.fn_lends.last().copied().unwrap_or(false) {
                                c.no_escape(last, "returning it");
                            }
                            c.no_escape_return(last, "returning it");
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
                    c.fixed_read(expr, ret);
                    c.returns(expr, &ty);
                    if !c.fn_lends.last().copied().unwrap_or(false) {
                        c.no_escape(expr, "returning it");
                    }
                    c.no_escape_return(expr, "returning it");
                }
                FnBody::None => {}
            }
        });
        self.fns.pop();
        self.fn_lends.pop();
        let gives = self.gives.pop().unwrap_or(false);
        if let Some((memo, is_array)) = memo {
            // Written `-> T`, or read off what the body gives.
            let unit = match &f.ret {
                Some(_) => **ret == Ty::Unit,
                None => !gives && !matches!(f.body, FnBody::None),
            };
            self.memo_fits(f, params, self_ty, generator, unit, memo, is_array);
        }
        self.pop_scope();
        if receiver.is_some() && matches!(self_ty, Some(Ty::Named { .. })) {
            self.pop_scope();
        }
        self.in_fn -= 1;
        self.moved = outer_moved;
        self.generics.truncate(outer);
        self.param_bounds.truncate(outer_bounds);
        self.row_bounds = saved_row_bounds;
    }
}

/// Where a fn's `@memo` is written, and whether it specifies `@memo(array)`.
fn memo_of(stmt: &Stmt) -> Option<(Span, bool)> {
    let memo = stmt.decorators.iter().find(|d| d.name.text == "memo")?;
    let is_array = memo
        .args
        .iter()
        .any(|a| a.name.is_none() && matches!(&a.value.kind, ExprKind::Name(n) if n == "array"));
    Some((memo.span, is_array))
}

impl Checker<'_> {
    /// The values of `fields(a, b, ..)` in a `for` head: the compile-time
    /// loop over a struct's or a tuple's fields (decision 28.1).
    pub(crate) fn reflected_values<'e>(
        &mut self,
        iter: &'e Expr,
    ) -> Option<&'e [cpl_syntax::ast::Arg]> {
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
            Some(Def::Enum { variants, .. }) => {
                Some(variants.iter().map(|(_, fs)| fs.clone()).collect())
            }
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
            let enum_mismatch = first.as_ref().is_some_and(|first| {
                !first.is_vague()
                    && !ty.is_vague()
                    && *first != ty
                    && (variant_shapes.is_some() || self.variant_payloads(&ty).is_some())
            });
            if enum_mismatch {
                let first = first.as_ref().unwrap();
                self.error(
                    codes::TYPE_MISMATCH,
                    v.value.span,
                    format!(
                        "`fields` walks values of one type together; this is a `{ty}`, the first is a `{first}`"
                    ),
                );
            }
            if first.is_none() {
                first = Some(ty.clone());
            }
            if let Some(variants) = self.variant_payloads(&ty) {
                variant_shapes = Some(variants);
                shapes.push(None);
                continue;
            }
            match self.reflect(&ty) {
                // Values of other types walk along when they have as many fields.
                Ok(Some(fields))
                    if shapes
                        .iter()
                        .flatten()
                        .next()
                        .is_some_and(|f: &Vec<(String, Ty)>| f.len() != fields.len()) =>
                {
                    self.error(
                        codes::TYPE_MISMATCH,
                        v.value.span,
                        format!(
                            "`fields` walks values with as many fields together; this `{ty}` has {}",
                            fields.len()
                        ),
                    );
                    shapes.push(None);
                }
                Ok(fields) => shapes.push(fields),
                Err(()) => {
                    if let Some(d) = self.error(
                        codes::TYPE_MISMATCH,
                        v.value.span,
                        format!(
                            "`fields` walks the fields of a struct or a tuple, not of a `{ty}`"
                        ),
                    ) {
                        d.help =
                            Some("pass a struct or tuple value: `for f in fields(p):`".to_string());
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

    fn context_record_fields(&mut self, name: &str) -> Option<Vec<(String, Ty)>> {
        if let Some(local) = self.local(name) {
            if local.keyword == "context" {
                if let Ty::Record(fields) = &local.ty {
                    return Some(fields.clone());
                }
            }
        }
        if let Some(Def::Value {
            keyword: "context",
            ty: Ty::Record(fields),
            ..
        }) = self.global(name).as_deref()
        {
            return Some(fields.clone());
        }
        if name == "io" {
            return Some(vec![
                (
                    "base".into(),
                    Ty::Range {
                        lo: Some(0),
                        hi: Some(1),
                        text: "0..=1".into(),
                    },
                ),
                ("sep".into(), Ty::Str),
                ("answers".into(), Ty::Tuple(vec![Ty::Str, Ty::Str])),
                ("precision".into(), Ty::Optional(Box::new(Ty::Int))),
            ]);
        }
        None
    }

    fn check_with(&mut self, ctx: &ast::Name, args: &[ast::Arg]) {
        if ctx.text == "checks" {
            const CHECK_FIELDS: &[&str] = &[
                "ranges", "overflow", "bounds", "unwrap", "div", "asserts", "reads",
            ];
            for arg in args {
                let Some(name) = &arg.name else {
                    self.error(
                        codes::TYPE_MISMATCH,
                        arg.value.span,
                        "arguments to `with checks(...)` must be named".to_string(),
                    );
                    continue;
                };
                if !CHECK_FIELDS.contains(&name.text.as_str()) {
                    self.error(
                        codes::UNKNOWN_FIELD,
                        name.span,
                        format!("context `checks` has no field `{}`", name.text),
                    );
                }
                if !matches!(&arg.value.kind, ExprKind::Name(v) if matches!(v.as_str(), "on" | "off"))
                {
                    self.error(
                        codes::TYPE_MISMATCH,
                        arg.value.span,
                        "expected `on` or `off` for `checks` context field".to_string(),
                    );
                }
            }
            return;
        }
        if ctx.text == "lint" {
            for arg in args {
                if arg.name.is_none() {
                    self.error(
                        codes::TYPE_MISMATCH,
                        arg.value.span,
                        "arguments to `with lint(...)` must be named".to_string(),
                    );
                    continue;
                }
                if !matches!(
                    &arg.value.kind,
                    ExprKind::Name(v) if matches!(v.as_str(), "allow" | "warn" | "deny")
                ) {
                    self.error(
                        codes::TYPE_MISMATCH,
                        arg.value.span,
                        "expected `allow`, `warn`, or `deny` for `lint` context field".to_string(),
                    );
                }
            }
            return;
        }
        let Some(fields) = self.context_record_fields(&ctx.text) else {
            self.error(
                codes::UNRESOLVED_NAME,
                ctx.span,
                format!("unknown context `{}`", ctx.text),
            );
            for arg in args {
                self.infer(&arg.value);
            }
            return;
        };
        for arg in args {
            if !context_constant(&arg.value) {
                let message = "a context value is a compile-time constant; pass a runtime value as an argument instead".to_string();
                self.error(codes::CONTEXT_NOT_CONSTANT, arg.value.span, message);
            }
            let Some(name) = &arg.name else {
                self.error(
                    codes::TYPE_MISMATCH,
                    arg.value.span,
                    format!("arguments to `with {}(...)` must be named", ctx.text),
                );
                self.infer(&arg.value);
                continue;
            };
            let Some((_, expected)) = fields.iter().find(|(n, _)| n == &name.text) else {
                self.error(
                    codes::UNKNOWN_FIELD,
                    name.span,
                    format!("context `{}` has no field `{}`", ctx.text, name.text),
                );
                self.infer(&arg.value);
                continue;
            };
            let got = self.infer(&arg.value);
            if !matches!(expected, Ty::Error | Ty::Unknown)
                && !matches!(got, Ty::Error | Ty::Unknown)
            {
                let target = match expected {
                    Ty::Optional(inner)
                        if !matches!(got, Ty::Optional(_))
                            && !matches!(arg.value.kind, ExprKind::None) =>
                    {
                        inner.as_ref()
                    }
                    other => other,
                };
                if let Ty::Range { lo, hi, .. } = target {
                    if let ExprKind::Int(n) = arg.value.kind {
                        let in_lo = lo.is_none_or(|l| n >= l);
                        let in_hi = hi.is_none_or(|h| n <= h);
                        if !in_lo || !in_hi {
                            self.error(
                                codes::TYPE_MISMATCH,
                                arg.value.span,
                                format!("expected `{expected}`, got `{n}`"),
                            );
                        }
                        continue;
                    }
                }
                self.expect_fits(&got, target, &arg.value, |_, got, want| {
                    format!("expected `{want}`, got `{got}`")
                });
            }
        }
    }
}

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

/// A literal, a level such as `off`, or a tuple of them. (R09.1.2)
fn context_constant(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Char(_)
        | ExprKind::Bool(_)
        | ExprKind::None => true,
        ExprKind::Str(pieces) => pieces
            .iter()
            .all(|p| matches!(p, cpl_syntax::ast::StrPiece::Text(_))),
        ExprKind::Name(name) => matches!(name.as_str(), "on" | "off" | "allow" | "warn" | "deny"),
        ExprKind::Unary { expr, .. } => context_constant(expr),
        ExprKind::Tuple(items) => items.iter().all(context_constant),
        _ => false,
    }
}

impl<'a> Checker<'a> {
    fn collect_returned_exprs<'b>(block: &'b ast::Block, out: &mut Vec<&'b Expr>) {
        for (i, stmt) in block.stmts.iter().enumerate() {
            match &stmt.kind {
                StmtKind::Return(Some(e)) | StmtKind::Lend { place: e, .. } => out.push(e),
                StmtKind::Expr(e) if i + 1 == block.stmts.len() => out.push(e),
                StmtKind::Expr(Expr {
                    kind:
                        ExprKind::If {
                            branches,
                            otherwise,
                        },
                    ..
                }) => {
                    for (_, b) in branches {
                        Self::collect_returned_exprs(b, out);
                    }
                    if let Some(b) = otherwise {
                        Self::collect_returned_exprs(b, out);
                    }
                }
                _ => {}
            }
        }
    }

    fn expr_root_name(mut e: &Expr) -> Option<&str> {
        loop {
            match &e.kind {
                ExprKind::Name(n) => return Some(n.as_str()),
                ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => e = base,
                ExprKind::Paren(inner) => e = inner,
                _ => return None,
            }
        }
    }

    fn expr_self_field(mut e: &Expr) -> Option<&str> {
        loop {
            match &e.kind {
                ExprKind::Field { base, name } if matches!(&base.kind, ExprKind::SelfValue) => {
                    return Some(name.text.as_str());
                }
                ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => e = base,
                ExprKind::Paren(inner) => e = inner,
                _ => return None,
            }
        }
    }

    fn check_cache_lend_body(&mut self, block: &ast::Block, cache_field: &str) {
        for stmt in &block.stmts {
            self.check_cache_lend_stmt(stmt, cache_field);
        }
    }

    fn check_cache_lend_stmt(&mut self, stmt: &Stmt, cache_field: &str) {
        match &stmt.kind {
            StmtKind::Expr(e) | StmtKind::Return(Some(e)) => {
                self.check_cache_lend_expr(e, cache_field);
            }
            StmtKind::Let { init: Some(e), .. } => self.check_cache_lend_expr(e, cache_field),
            StmtKind::Assign { target, value, .. } => {
                self.check_cache_lend_expr(target, cache_field);
                self.check_cache_lend_expr(value, cache_field);
            }
            StmtKind::For { iter, body, .. } => {
                self.check_cache_lend_expr(iter, cache_field);
                self.check_cache_lend_body(body, cache_field);
            }
            StmtKind::While { body, .. } => {
                self.check_cache_lend_body(body, cache_field);
            }
            _ => {}
        }
    }

    fn check_cache_lend_expr(&mut self, expr: &Expr, cache_field: &str) {
        match &expr.kind {
            ExprKind::Call { callee, args } => {
                let bad_self_method = matches!(
                    &callee.kind,
                    ExprKind::Field { base, .. }
                        if matches!(&base.kind, ExprKind::SelfValue)
                );
                let bad_fn_param = match &callee.kind {
                    ExprKind::Name(n) => {
                        self.local(n).is_some_and(|l| matches!(l.ty, Ty::Fn { .. }))
                    }
                    _ => false,
                };
                if bad_self_method || bad_fn_param {
                    self.error(
                        codes::USED_WHILE_LENT,
                        expr.span,
                        format!(
                            "cannot call `{}` while `@cache` field `self.{cache_field}` is lent",
                            self.quote(callee.span)
                        ),
                    );
                }
                self.check_cache_lend_expr(callee, cache_field);
                for a in args {
                    self.check_cache_lend_expr(&a.value, cache_field);
                }
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                self.check_cache_lend_expr(lhs, cache_field);
                self.check_cache_lend_expr(rhs, cache_field);
            }
            ExprKind::Compare { first, rest } => {
                self.check_cache_lend_expr(first, cache_field);
                for (_, e) in rest {
                    self.check_cache_lend_expr(e, cache_field);
                }
            }
            ExprKind::Unary { expr: inner, .. }
            | ExprKind::Paren(inner)
            | ExprKind::Try(inner)
            | ExprKind::Unwrap(inner) => self.check_cache_lend_expr(inner, cache_field),
            ExprKind::If {
                branches,
                otherwise,
            } => {
                for (c, b) in branches {
                    if let ast::Cond::Expr(e) | ast::Cond::Const(e) = c {
                        self.check_cache_lend_expr(e, cache_field);
                    }
                    self.check_cache_lend_body(b, cache_field);
                }
                if let Some(b) = otherwise {
                    self.check_cache_lend_body(b, cache_field);
                }
            }
            _ => {}
        }
    }
}

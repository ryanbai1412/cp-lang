//! The program's entry points: `main`, `@!multitest`, a `@test` fn, an
//! `@alt` or `@check` fn run as the program, and gathering the result.

use super::*;
use cpl_diag::msgs;

impl Lowerer<'_> {
    /// Lowers every fn the program reaches and gathers the [`Program`].
    /// `whole` is a run of the file itself: every fn of the file is lowered,
    /// called or not, so each one's errors are reported.
    pub(in crate::lower) fn finish(
        self,
        analysis: &Analysis,
        whole: bool,
    ) -> Result<Program, Vec<Diagnostic>> {
        let mut lw = self;
        if whole {
            for id in 0..lw.sigs.len() {
                // A fn with no body is the judge's: there is nothing to lower
                // until something calls it.
                let bodyless = matches!(lw.sigs[id].decl.body, ast::FnBody::None);
                if lw.sigs[id].mod_id == 0 && lw.sigs[id].owner.is_none() && !bodyless {
                    lw.queue_fn(id);
                }
            }
        }
        loop {
            while let Some(id) = lw.queued_fns.pop() {
                lw.function(id);
            }
            let mut newly_called = Vec::new();
            collect_block_calls(&lw.main_body, &mut newly_called);
            collect_block_calls(&lw.setup_body, &mut newly_called);
            for s in &lw.preamble_stmts {
                collect_stmt_calls(s, &mut newly_called);
            }
            for c in &lw.closures {
                collect_block_calls(&c.body, &mut newly_called);
            }
            for body in lw.bodies.iter().flatten() {
                collect_block_calls(&body.body, &mut newly_called);
            }
            for s in &lw.structs {
                for hook in [
                    s.def.eq_fn,
                    s.def.hash_fn,
                    s.def.ord.map(OrdHook::id),
                    s.def.show_fn,
                ]
                .into_iter()
                .flatten()
                {
                    newly_called.push(hook);
                }
            }
            newly_called.extend(lw.enums.iter().filter_map(|e| e.show_fn));
            let mut added = false;
            for fid in newly_called {
                if lw.queued_set.insert(fid) {
                    lw.queued_fns.push(fid);
                    added = true;
                }
            }
            if !added {
                break;
            }
        }
        // A `@memo` table starts unset, and anew in each test case
        // (decision 20.2).
        if !lw.memo_resets.is_empty() {
            let unset: Vec<Stmt> = lw
                .memo_resets
                .iter()
                .map(|ready| {
                    let no = lw.mk(ExprKind::Bool(false), Ty::Bool, Span::default());
                    Stmt::Assign(ready.clone(), AssignOp::Set, no)
                })
                .collect();
            if lw.multitest {
                lw.setup_body.stmts.splice(0..0, unset.iter().cloned());
            }
            lw.main_body.stmts.splice(0..0, unset);
        }
        // The initializers of imported modules' consts run first, once.
        if !lw.preamble_stmts.is_empty() {
            let preamble = std::mem::take(&mut lw.preamble_stmts);
            let first = if lw.multitest {
                &mut lw.setup_body
            } else {
                &mut lw.main_body
            };
            first.stmts.splice(0..0, preamble);
        }
        // A library domain is sized after every definition of a domain it
        // reads.
        let mut main = std::mem::take(&mut lw.main_body);
        let mut setup = std::mem::take(&mut lw.setup_body);
        let mut bodies = std::mem::take(&mut lw.bodies);
        let mut closures = std::mem::take(&mut lw.closures);
        lw.size_library_domains(&mut main, true, &lw.main_locals);
        lw.size_library_domains(&mut setup, true, &lw.main_locals);
        for f in bodies.iter_mut().flatten() {
            lw.size_library_domains(&mut f.body, false, &f.locals);
        }
        for c in &mut closures {
            lw.size_library_domains(&mut c.body, false, &c.locals);
        }
        (lw.main_body, lw.setup_body, lw.bodies, lw.closures) = (main, setup, bodies, closures);
        // A mistake in the user's code often breaks the library code it asks
        // for as well. The user's own mistake is the one to show.
        if lw.diags.iter().any(|d| !lw.in_library.contains(&d.message)) {
            let in_library = std::mem::take(&mut lw.in_library);
            lw.diags.retain(|d| !in_library.contains(&d.message));
        }
        if !lw.diags.is_empty() {
            cpl_diag::sort_by_key(&mut lw.diags, |d| d.span.order());
            // One error per place, as `Sink::push` keeps for the checker.
            lw.diags.dedup_by(|b, a| {
                a.span == b.span && (a.code == b.code || a.severity == b.severity)
            });
            return Err(lw.diags);
        }
        let (judge_fns, entry_fns, cpp_bodies) = lw.opaque_fns(analysis);
        if !lw.diags.is_empty() {
            return Err(lw.diags);
        }
        // An error value with no error to say why would reach both engines
        // as no value at all, so the program would run and print wrong.
        if let Some(at) = lw.unreported_error() {
            return Err(vec![msgs::ir::INTERNAL_LOWERED_TO_NOTHING.error(at, &[])]);
        }
        // Ensure any unlowered non-generic fns from imported modules (unused) don't panic:
        // keep only lowered fns or fill unused bodies with empty blocks, and deduplicate names.
        let said: Vec<String> = (0..lw.domains.len()).map(|d| lw.domain_said(d)).collect();
        let struct_shown = lw
            .shown_structs(&said)
            .into_iter()
            .map(|s| s.name)
            .collect();
        let mut used_fn_names: HashSet<String> = HashSet::new();
        let (library_fns, fn_display) = lw.fn_names();
        let fns = lw
            .bodies
            .into_iter()
            .enumerate()
            .map(|(i, body)| {
                let mut f = body.unwrap_or_else(|| Function {
                    name: lw.sigs[i].name.clone(),
                    params: Vec::new(),
                    ret: Ty::Unit,
                    locals: Vec::new(),
                    body: Block::default(),
                });
                if !used_fn_names.insert(f.name.clone()) {
                    f.name = format!("{}_{i}", f.name);
                    used_fn_names.insert(f.name.clone());
                }
                f
            })
            .collect();
        cpl_diag::sort_by_key(&mut lw.warnings, |d| d.span.order());
        lw.warnings.dedup();
        for (&gid, info) in &lw.global_ranges {
            if let Some(g) = lw.globals.get(gid).filter(|g| is_narrowable_elem_ty(&g.ty)) {
                if let Some(layout) = info.narrow_elem_layout(&g.ty) {
                    lw.narrow_elems.insert(ElemLoc::Global(gid), layout);
                }
            }
        }
        let products = (0..lw.domains.len())
            .map(|d| {
                let parts = lw.product_domains.get(&d)?;
                Some(
                    parts
                        .iter()
                        .map(|(name, f)| (name.clone(), f.ty()))
                        .collect(),
                )
            })
            .collect();
        let lines = &lw.mods[0].lines;
        let user_line_starts = (0..lines.line_count()).map(|l| lines.line_start(l));
        let program = Program {
            user_line_starts: user_line_starts.collect(),
            warnings: lw.warnings,
            struct_shown,
            unordered_maps: Vec::new(),
            wide_domains: Vec::new(),
            fixed_lens: Vec::new(),
            byte_domains: Vec::new(),
            unordered_sets: Vec::new(),
            unreached_closures: Default::default(),
            structs: lw.structs.iter().map(|s| s.def.clone()).collect(),
            enums: lw.enums,
            domains: lw.domains,
            products,
            globals: lw.globals,
            fns,
            library_fns,
            fn_display,
            closures: lw.closures,
            main: lw.main_body,
            main_locals: lw.main_locals,
            setup: lw.setup_body,
            multitest: lw.multitest,
            multitest_span: lw.multitest_span,
            until_eof: lw.until_eof,
            interactive: lw.interactive,
            announce_tests: lw.announce_tests,
            judge_fns,
            cpp_bodies,
            entry_fns,
            judge_tys: lw.judge_tys,
            raw_lists: lw.raw_lists,
            narrow_elems: lw.narrow_elems,
            written_back: {
                let mut spans = lw.written_back;
                cpl_diag::sort_by_key(&mut spans, |s| s.order());
                spans.dedup();
                spans
            },
        };
        let escapes = crate::pushed::escaping_yields(&program);
        escapes.is_empty().then_some(program).ok_or(escapes)
    }
}

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn main(&mut self) {
        self.cur_mod = 0;
        self.cx = FnCx {
            is_main: true,
            ret: Some(Ty::Unit),
            in_body: true,
            ..FnCx::default()
        };
        let mut out = Vec::new();
        let items: Vec<(u32, &'a ast::Stmt)> = self.mod_items(0).collect();
        let hoisted = self.hoist_constant_domains(&items, &mut out);
        for (idx, &(offset, stmt)) in items.iter().enumerate() {
            self.offset = offset;
            // `@!multitest` divides the file whatever item it sits on.
            if let Some(d) = stmt.decorators.iter().find(|d| d.name.text == "multitest") {
                self.divide(d, &mut out);
            }
            // Skip `@test` functions in normal `cpl run`/`cpl build`.
            if hoisted.contains(&idx) || stmt.decorators.iter().any(|d| d.name.text == "test") {
                continue;
            }
            if matches!(stmt.kind, ast::StmtKind::Let { ty: None, .. }) {
                let rest: Vec<ast::Stmt> = items[idx + 1..]
                    .iter()
                    .filter(|(_, s)| !s.decorators.iter().any(|d| d.name.text == "test"))
                    .map(|(_, s)| (*s).clone())
                    .collect();
                self.stmt_with_rest(stmt, &rest, &mut out);
            } else {
                self.stmt(stmt, &mut out);
            }
        }
        let cx = std::mem::take(&mut self.cx);
        self.record_narrow_locals(ScopeId::Main, &cx.locals, &cx.local_ranges);
        self.main_body = Block { stmts: out };
        self.main_locals = cx.locals;
    }

    /// `@!multitest`: what was lowered so far runs once, and the rest of
    /// the file once per test case (decision 20.1). A bare `@multitest` is
    /// the older spelling of the same divider.
    pub(in crate::lower) fn divide(&mut self, decorator: &ast::Decorator, out: &mut Vec<Stmt>) {
        if self.multitest {
            self.report(
                decorator.span,
                msgs::ir::UNEXPECTED_TOKEN_A_SECOND_MULTITEST,
                &[],
            );
            return;
        }
        self.multitest = true;
        self.multitest_span = self.span(decorator.span);
        for arg in &decorator.args {
            match &arg.value.kind {
                ast::ExprKind::Name(name) if arg.name.is_none() && name == "until_eof" => {
                    self.until_eof = true;
                }
                _ => self.report(
                    arg.value.span,
                    msgs::ir::UNEXPECTED_TOKEN_MULTITEST_TAKES_NOTHING_OR,
                    &[],
                ),
            }
        }
        self.setup_body = Block {
            stmts: std::mem::take(out),
        };
    }

    pub(in crate::lower) fn main_for_test(&mut self, test_name: &str) {
        self.cur_mod = 0;
        self.cx = FnCx {
            is_main: true,
            ret: Some(Ty::Unit),
            in_body: true,
            ..FnCx::default()
        };
        let mut out = Vec::new();
        let items: Vec<(u32, &'a ast::Stmt)> = self.mod_items(0).collect();
        for (offset, stmt) in items {
            self.offset = offset;
            // Under `@!multitest` the judge runs once per test case, as
            // the solution does, and tells it how many there are.
            if self.judge {
                for decorator in &stmt.decorators {
                    if decorator.name.text == "multitest" {
                        self.divide(decorator, &mut out);
                        self.announce_tests = !self.until_eof;
                    }
                }
            }
            // Run top-level constants/lets/domains/impl Index initializers before the test.
            match &stmt.kind {
                ast::StmtKind::Const { generics, .. } if generics.is_empty() => {
                    self.stmt(stmt, &mut out);
                }
                // The solution's own lines are not the judge's to run.
                ast::StmtKind::Let { .. } | ast::StmtKind::Domain { .. } if self.judge => {}
                ast::StmtKind::Let { .. }
                | ast::StmtKind::Domain { .. }
                | ast::StmtKind::Impl { .. } => {
                    self.stmt(stmt, &mut out);
                }
                _ => {}
            }
        }
        if let Some(Sym::Fn(fn_id)) = self.mod_syms[0].get(test_name).copied() {
            self.queue_fn(fn_id);
            let ret = self.ret_of(fn_id, Span::default());
            // An `@interactor` is handed the solution: `sol`.
            let mutable: Vec<bool> = self.sigs[fn_id].params.iter().map(|p| p.mutable).collect();
            let args = mutable
                .into_iter()
                .map(|mutable| {
                    if !mutable {
                        return Arg::Value(self.mk(ExprKind::Unit, Ty::Unit, Span::default()));
                    }
                    let slot = self.new_slot("", Ty::Unit);
                    out.push(Stmt::Let(slot, None));
                    Arg::Mut(Place {
                        kind: PlaceKind::Local(slot),
                        ty: Ty::Unit,
                        span: Span::default(),
                    })
                })
                .collect();
            let call = Expr {
                kind: ExprKind::Call(fn_id, args),
                ty: ret,
                span: Span::default(),
            };
            out.push(Stmt::Expr(call));
        } else {
            self.report(
                Span::default(),
                msgs::ir::UNRESOLVED_NAME_NO_TEST_FN_NAMED,
                &[&test_name],
            );
        }
        let cx = std::mem::take(&mut self.cx);
        self.record_narrow_locals(ScopeId::Main, &cx.locals, &cx.local_ranges);
        self.main_body = Block { stmts: out };
        self.main_locals = cx.locals;
    }

    /// Whether `text` calls `read()`.
    pub(in crate::lower) fn reads_input(text: &str) -> bool {
        text.match_indices("read").any(|(at, _)| {
            matches!(text.as_bytes().get(at + 4), Some(b'(' | b'<'))
                && !text[..at]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.')
        })
    }

    /// A `@alt(program)` or `@check` fn as the program: the file's input
    /// header (every top-level line down to the last `read()`), so the fn
    /// sees the input by the solution's names in single-test and multitest
    /// files alike, then a call of the fn. Below the header only constants
    /// are kept: the rest is the solution's own work.
    pub(in crate::lower) fn main_for_aside(&mut self, name: &str) {
        self.cur_mod = 0;
        self.cx = FnCx {
            is_main: true,
            ret: Some(Ty::Unit),
            in_body: true,
            ..FnCx::default()
        };
        let mut out = Vec::new();
        let items: Vec<(u32, &'a ast::Stmt)> = self.mod_items(0).collect();
        let is_item = |stmt: &ast::Stmt| {
            matches!(
                stmt.kind,
                ast::StmtKind::Fn(_) | ast::StmtKind::Struct(_) | ast::StmtKind::Impl { .. }
            )
        };
        let own_reads = items.iter().any(|(offset, stmt)| {
            self.offset = *offset;
            matches!(&stmt.kind, ast::StmtKind::Fn(decl) if decl.name.text == name)
                && Self::reads_input(&self.source(stmt.span))
        });
        let size = match self.aside {
            Some(Aside::Gen(size)) => Some(size),
            _ => None,
        };
        let header = if let Some(Aside::Read(line)) = self.aside {
            let text = self.mods[0].text.clone();
            let line_of =
                |at: u32| text[..(at as usize).min(text.len())].matches('\n').count() as u32 + 1;
            items
                .iter()
                .filter(|(offset, stmt)| line_of(stmt.span.shift(*offset).start) < line)
                .count()
        } else if own_reads || size.is_some() {
            self.shared_reads(&items, name, size.is_some())
        } else {
            items
                .iter()
                .rposition(|(offset, stmt)| {
                    self.offset = *offset;
                    !is_item(stmt) && Self::reads_input(&self.source(stmt.span))
                })
                .map_or(0, |last| last + 1)
        };
        for (idx, &(offset, stmt)) in items.iter().enumerate() {
            self.offset = offset;
            if stmt.decorators.iter().any(|d| d.name.text == "test") {
                continue;
            }
            for decorator in &stmt.decorators {
                if decorator.name.text == "multitest" && !matches!(self.aside, Some(Aside::Gen(_)))
                {
                    self.divide(decorator, &mut out);
                }
            }
            let constant = call::fixed_domain(stmt)
                || matches!(&stmt.kind, ast::StmtKind::Const { generics, .. } if generics.is_empty());
            if idx < header || constant || is_item(stmt) {
                self.stmt(stmt, &mut out);
                continue;
            }
            match &stmt.kind {
                ast::StmtKind::Let { pat, .. } => Self::pattern_names(pat, &mut self.tail_names),
                ast::StmtKind::Domain { name, .. } => {
                    self.tail_names.insert(name.text.clone());
                }
                _ => {}
            }
        }
        if let Some(Sym::Fn(fn_id)) = self.mod_syms[0].get(name).copied() {
            self.queue_fn(fn_id);
            let at = self.sigs[fn_id].decl.name.span;
            let item_offset = std::mem::replace(&mut self.offset, self.sigs[fn_id].offset);
            let params: Vec<Ty> = self.sigs[fn_id]
                .params
                .iter()
                .map(|p| p.ty.clone())
                .collect();
            let args = match (size, params.as_slice()) {
                (_, []) => Vec::new(),
                (Some(size), [Ty::Int]) => vec![Arg::Value(Expr {
                    kind: ExprKind::Int(size),
                    ty: Ty::Int,
                    span: Span::default(),
                })],
                (Some(_), _) => {
                    let message = cpl_diag::msgs::ir::GEN_FN_TAKES_NOTHING.at(&[]);
                    self.report_said(at, message);
                    Vec::new()
                }
                (None, _) => {
                    let kind = if self.aside == Some(Aside::Brute) {
                        "@alt(program)"
                    } else {
                        "@check"
                    };
                    self.report(at, msgs::ir::WRONG_ARG_COUNT_A_FN_TAKES_NO, &[&kind]);
                    Vec::new()
                }
            };
            let span = self.span(at);
            self.offset = item_offset;
            let ret = self.ret_of(fn_id, Span::default());
            let call = Expr {
                kind: ExprKind::Call(fn_id, args),
                ty: ret,
                span: Span::default(),
            };
            if let Some(Aside::Read(_)) = self.aside {
                let printed = self.fmt_arg(call, crate::FLOAT_DIGITS, Span::default());
                let print = self.mk(
                    ExprKind::Print(printed.into_iter().collect(), None),
                    Ty::Unit,
                    Span::default(),
                );
                out.push(Stmt::Expr(print));
            } else if self.aside == Some(Aside::Checker) && call.ty == Ty::Bool {
                // `@check fn ok() -> Bool` rejects by returning `false`, as a failing
                // `assert` does. Not an `assert`: its report would run the fn again.
                let rejected = format!("`{name}()` returned false: the check rejects this output");
                out.push(Stmt::If(
                    call,
                    Block { stmts: Vec::new() },
                    Block {
                        stmts: vec![Stmt::Trap(rejected, span)],
                    },
                ));
            } else {
                out.push(Stmt::Expr(call));
            }
        }
        let cx = std::mem::take(&mut self.cx);
        self.record_narrow_locals(ScopeId::Main, &cx.locals, &cx.local_ranges);
        self.main_body = Block { stmts: out };
        self.main_locals = cx.locals;
    }

    /// Where an error value is left in the lowered program, which only a
    /// reported error may leave.
    fn unreported_error(&self) -> Option<Span> {
        let mut at = None;
        let mut seen = |e: &Expr| {
            if matches!(e.kind, ExprKind::Error) {
                at.get_or_insert(e.span);
            }
        };
        let blocks = [&self.main_body, &self.setup_body]
            .into_iter()
            .chain(self.bodies.iter().flatten().map(|f| &f.body))
            .chain(self.closures.iter().map(|c| &c.body));
        for block in blocks {
            crate::walk::deep_exprs(block, &mut seen);
        }
        at
    }
}

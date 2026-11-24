//! What code changes outside itself: the outer names a fn, lambda or
//! call writes and whether it reads input or writes output, and what that
//! means for a call's arguments (R05.9).

use super::*;
use cpl_diag::msgs;

/// What a fn touches outside itself, read off its source: the outer names
/// it changes and reads, and the fns it calls. A name is outer unless the
/// fn declares it (a parameter, a `let`, a loop variable); a nested fn is
/// part of the body, so what it touches is charged to the fn around it.
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Effects {
    pub writes: Vec<String>,
    pub reads: Vec<String>,
    pub calls: Vec<String>,
    /// `reads input` or `writes output`, if the fn or one it calls does.
    pub io: Option<&'static str>,
}

/// What calling the prelude fn `name` does to the input or the output.
pub(crate) fn io_of(name: &str) -> Option<&'static str> {
    match name {
        "read" | "read_line" | "read_output" => Some("reads input"),
        "print" | "flush" | "ask" => Some("writes output"),
        _ => None,
    }
}

impl Effects {
    /// Adds what a fn this one calls touches.
    pub(crate) fn absorb(&mut self, other: &Effects) {
        for (into, from) in [
            (&mut self.writes, &other.writes),
            (&mut self.reads, &other.reads),
        ] {
            for name in from {
                if !into.contains(name) {
                    into.push(name.clone());
                }
            }
        }
        self.io = self.io.or(other.io);
    }

    /// How the fn uses the outer name `name`, if it does: `changes` or
    /// `reads`.
    pub(super) fn does(&self, name: &str) -> Option<&'static str> {
        if self.writes.iter().any(|n| n == name) {
            Some("changes")
        } else if self.reads.iter().any(|n| n == name) {
            Some("reads")
        } else {
            None
        }
    }
}

#[derive(Default)]
pub(super) struct Touched {
    pub(super) declared: HashSet<String>,
    pub(super) written: Written,
    pub(super) read: Vec<String>,
    pub(super) calls: Vec<String>,
}

impl Visitor for Touched {
    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Assign { target, .. } => self.written.add(root_name(target)),
            StmtKind::Fn(f) => {
                self.declared.insert(f.name.text.clone());
                self.declared
                    .extend(f.params.iter().map(|p| p.name.text.clone()));
            }
            _ => {}
        }
    }

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Call { callee, args } => {
                for arg in args.iter().filter(|a| a.mutable) {
                    self.written.add(root_name(&arg.value));
                }
                match &callee.kind {
                    ExprKind::Field { base, name } if mutating_call(&name.text, args) => {
                        self.written.add(root_name(base));
                    }
                    ExprKind::Name(name) if !self.calls.contains(name) => {
                        self.calls.push(name.clone());
                    }
                    _ => {}
                }
            }
            ExprKind::Name(name) if !self.read.contains(name) => self.read.push(name.clone()),
            _ => {}
        }
    }

    fn pattern(&mut self, pat: &ast::Pattern) {
        let mut names = Written::default();
        names.pattern(pat);
        self.declared.extend(names.0);
    }
}

pub(crate) fn lambda_effects(params: &[ast::Pattern], body: &Expr) -> Effects {
    let mut touched = Touched::default();
    for p in params {
        let mut names = Names::default();
        names.pattern(p);
        touched.declared.extend(names.0);
    }
    walk_expr(body, &mut touched);
    let Touched {
        declared,
        written,
        read,
        calls,
    } = touched;
    let outer = |names: Vec<String>| -> Vec<String> {
        names
            .into_iter()
            .filter(|n| !declared.contains(n))
            .collect()
    };
    let calls = outer(calls);
    Effects {
        writes: outer(written.0),
        reads: outer(read),
        io: calls.iter().find_map(|name| io_of(name)),
        calls,
    }
}

pub(crate) fn own_effects(f: &ast::FnDecl) -> Effects {
    let mut touched = Touched::default();
    touched
        .declared
        .extend(f.params.iter().map(|p| p.name.text.clone()));
    match &f.body {
        ast::FnBody::Block(block) => walk_block(block, &mut touched),
        ast::FnBody::Expr(expr) => walk_expr(expr, &mut touched),
        ast::FnBody::None => {}
    }
    touched.effects()
}

impl Touched {
    pub(super) fn effects(self) -> Effects {
        let Touched {
            declared,
            written,
            read,
            calls,
        } = self;
        let outer = |names: Vec<String>| -> Vec<String> {
            names
                .into_iter()
                .filter(|n| !declared.contains(n))
                .collect()
        };
        let calls = outer(calls);
        Effects {
            writes: outer(written.0),
            reads: outer(read),
            io: calls.iter().find_map(|name| io_of(name)),
            calls,
        }
    }
}

impl<'a> Checker<'a> {
    /// The outer names evaluating `e` may change: what it assigns through
    /// `mut` arguments and mutating methods, and what the fns it calls
    /// change.
    pub(super) fn changed_by(&mut self, e: &Expr) -> Vec<String> {
        let mut touched = Touched::default();
        walk_expr(e, &mut touched);
        let mut writes = touched.written.0;
        for name in touched.calls {
            if !self.names_fn_item(&name) {
                continue;
            }
            for write in &self.env.effects(&name).writes {
                if !writes.contains(write) {
                    writes.push(write.clone());
                }
            }
        }
        writes
    }

    /// True if `arg` reaches the callee as the place itself: a number or
    /// a position read from a place is copied before the call; anything
    /// bigger, and anything passed `mut`, is passed by reference.
    pub(super) fn by_reference(&mut self, arg: &Arg) -> bool {
        if arg.mutable {
            return true;
        }
        let ty = self.infer_quietly(&arg.value);
        matches!(
            ty,
            Ty::Array { .. } | Ty::Named { .. } | Ty::Str | Ty::Tuple(_) | Ty::Record(_)
        )
    }

    /// `debug(q.pop())`: `cpl build` leaves `debug` out of the C++ it
    /// writes, so an argument that changes a variable or reads or writes
    /// would do so in `cpl run` and not on the judge.
    pub(crate) fn debug_changes(&mut self, callee: &Expr, args: &[Arg]) {
        let prelude = matches!(&callee.kind, ExprKind::Name(name)
            if name == "debug" && self.local(name).is_none() && self.global(name).is_none());
        if !prelude {
            return;
        }
        for arg in args {
            let Some((at, what)) = self.change_in(&arg.value) else {
                continue;
            };
            if let Some(diag) =
                self.report(at, msgs::check::DEBUG_CHANGES_DEBUG_IS_LEFT_OUT, &[&what])
            {
                diag.help = Some("do it in a statement before, and show the result: `let x = ..` then `debug(x)`".to_string());
            }
        }
    }

    /// The first call in `e` that changes a variable or reads or writes,
    /// with what it does.
    pub(super) fn change_in(&mut self, e: &Expr) -> Option<(Span, String)> {
        struct Calls(Vec<Expr>);
        impl Visitor for Calls {
            fn expr(&mut self, e: &Expr) {
                if matches!(e.kind, ExprKind::Call { .. }) {
                    self.0.push(e.clone());
                }
            }
        }
        let mut calls = Calls(Vec::new());
        walk_expr(e, &mut calls);
        for call in &calls.0 {
            let ExprKind::Call { callee, args } = &call.kind else {
                continue;
            };
            if let Some(arg) = args.iter().find(|a| a.mutable) {
                let name = root_name(&arg.value).unwrap_or("its argument");
                return Some((call.span, format!("changes `{name}`, passed `mut`")));
            }
            match &callee.kind {
                ExprKind::Field { base, name } => {
                    let base_ty = self.infer_quietly(base);
                    if self.own_mutator(&base_ty, &name.text)
                        || self.row_mutator(&base_ty, &name.text, args)
                    {
                        let root = root_name(base).unwrap_or("what it is called on");
                        return Some((call.span, format!("changes `{root}`")));
                    }
                }
                ExprKind::Name(name) if self.local(name).is_none() => {
                    let global = self.global(name);
                    let Some(Def::Fn { .. }) = global.as_deref() else {
                        if let Some(what) = global.is_none().then(|| io_of(name)).flatten() {
                            return Some((call.span, what.to_string()));
                        }
                        continue;
                    };
                    let effects = self.env.effects(name);
                    if let Some(changed) = effects.writes.first() {
                        return Some((
                            call.span,
                            format!("calls `{name}`, which changes `{changed}`"),
                        ));
                    }
                    if let Some(what) = effects.io {
                        return Some((call.span, format!("calls `{name}`, which {what}")));
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// `xs.sort(by: key)` sorts `xs` where it is, so the key can't reach
    /// `xs` while it runs: not by name in a closure, and not through a fn
    /// that reads or changes it. (`xs.sorted(by:)` sorts a copy.)
    pub(crate) fn sort_key_uses(&mut self, callee: &Expr, args: &[Arg]) {
        let ExprKind::Field { base, name } = &callee.kind else {
            return;
        };
        if let ExprKind::Name(n) = &base.kind {
            if matches!(self.local(n).map(|b| &b.ty), Some(Ty::Domain(_)))
                || self.global_is(n, |d| matches!(d, Def::Domain { .. }))
            {
                return;
            }
        }
        let (Some(root), Some(key)) = (
            root_name(base).filter(|_| name.text == "sort"),
            args.iter().find(|a| {
                a.name
                    .as_ref()
                    .is_some_and(|n| matches!(n.text.as_str(), "by" | "cmp"))
            }),
        ) else {
            return;
        };
        struct Names(Vec<String>);
        impl Visitor for Names {
            fn expr(&mut self, e: &Expr) {
                if let ExprKind::Name(name) = &e.kind {
                    self.0.push(name.clone());
                }
            }
        }
        let mut names = Names(Vec::new());
        walk_expr(&key.value, &mut names);
        let outer = self.local(root).is_none();
        let what = names.0.iter().find_map(|used| {
            if used == root {
                return Some(format!("uses `{root}`"));
            }
            let global_fn = self.names_fn_item(used);
            match (outer && global_fn).then(|| self.env.effects(used)) {
                Some(effects) => effects
                    .does(root)
                    .map(|what| format!("calls `{used}`, which {what} `{root}`")),
                None => None,
            }
        });
        let Some(what) = what else {
            return;
        };
        let opt = key.name.as_ref().map(|n| n.text.as_str()).unwrap_or("by");
        let role = if opt == "cmp" { "comparator" } else { "key" };
        let message =
            cpl_diag::msgs::check::SORT_WHILE_BEING_SORTED.at(&[&role, &root, &opt, &what, &root]);
        if let Some(diag) = self.report_said(key.value.span, message) {
            diag.help = Some(format!(
                "sort a copy, whose key may use the list: `{root} = {root}.sorted(by: ..)`"
            ));
        }
    }

    /// A fn that touches a `var` directly, not through a parameter, is one
    /// more user of it for the whole call: what is known about the names
    /// it changes is forgotten, it can't run while one of them is lent,
    /// and it can't also be handed one of them as an argument.
    pub(crate) fn call_effects(&mut self, call: &Expr, callee: &Expr, args: &[Arg]) {
        let ExprKind::Name(name) = &callee.kind else {
            return;
        };
        let (effects, is_local_fn) = if let Some(eff) = self.local_effects.get(name).cloned() {
            (std::sync::Arc::new(eff), true)
        } else if self.local(name).is_some() {
            return;
        } else {
            if !matches!(
                self.global(name).as_deref(),
                Some(Def::Fn { .. } | Def::Value { .. })
            ) {
                return;
            }
            (self.env.effects(name), false)
        };
        // A local of the same name is another variable (except for a local fn capturing it).
        let does = |c: &Self, root: &str| {
            if is_local_fn {
                effects.does(root)
            } else {
                match c.local(root) {
                    Some(_) => None,
                    None => effects.does(root),
                }
            }
        };
        let writes: Vec<String> = effects
            .writes
            .iter()
            .filter(|n| self.local(n).is_none())
            .cloned()
            .collect();
        self.drop_facts(&writes);

        for index in 0..self.lends.holds.len() {
            let hold = &self.lends.holds[index];
            if hold.is_capture {
                continue;
            }
            let Some(what) = does(self, &hold.root) else {
                continue;
            };
            if what == "reads" && !hold.mutable {
                continue;
            }
            let (root, lent_to, mutable, at, until, in_loop) = (
                hold.root.clone(),
                hold.name.clone(),
                hold.mutable,
                hold.at,
                hold.until,
                hold.in_loop,
            );
            if in_loop {
                if let Some(diag) = self.report(
                    call.span,
                    msgs::check::USED_WHILE_LENT_CAN_T_BE_CALLED,
                    &[&name, &what, &root, &root],
                ) {
                    diag.labels.push(cpl_diag::Label {
                        span: at,
                        message: format!("the loop goes over `{root}`"),
                    });
                    diag.help = Some(format!(
                        "loop over positions instead (`for i in 0..{root}.len`), or over a copy (`for x in {root}.clone()`)"
                    ));
                }
                return;
            }
            if let Some(diag) = self.report(
                call.span,
                msgs::check::USED_WHILE_LENT_CAN_T_BE_CALLED_2,
                &[
                    &name,
                    &what,
                    &root,
                    &root,
                    &if mutable { "mutably " } else { "" },
                    &lent_to,
                ],
            ) {
                if at != ELSEWHERE {
                    diag.labels.push(cpl_diag::Label {
                        span: at,
                        message: format!("lent to `{lent_to}` here"),
                    });
                }
                if until != ELSEWHERE {
                    diag.labels.push(cpl_diag::Label {
                        span: until,
                        message: format!("`{lent_to}` is last used here"),
                    });
                }
                diag.help = Some(format!(
                    "finish with `{lent_to}` first, or re-project after the call"
                ));
            }
            return;
        }

        for (k, arg) in args.iter().enumerate() {
            let Some(place) = self.place(&arg.value) else {
                continue;
            };
            if !self.by_reference(arg) {
                continue;
            }
            let root = place.root;
            // Handed over `mut`, the place is the callee's to change, and
            // it would also reach it by name. Without `mut` it is handed
            // a value, which does not move when the callee changes the
            // `var` it came from.
            if let Some(what) = does(self, &root) {
                if arg.mutable {
                    if let Some(diag) = self.report(
                        arg.value.span,
                        msgs::check::LENT_TWICE_CAN_T_BE_PASSED,
                        &[&show(&arg.value), &name, &name, &what, &root],
                    ) {
                        diag.help = Some(format!(
                            "pass everything `{name}` needs from `{root}` as parameters, or copy the value out first"
                        ));
                    }
                }
            }
            self.no_capture_of_mut_arg(args, k, &root, name);
            // Another argument's own calls must not change what this one
            // lends.
            for (j, other) in args.iter().enumerate() {
                if j == k || !self.changed_by(&other.value).contains(&root) {
                    continue;
                }
                if let Some(diag) = self.report(
                    other.value.span,
                    msgs::check::LENT_TWICE_CHANGES_WHILE_IS_LENT,
                    &[&show(&other.value), &root, &show(&arg.value)],
                ) {
                    diag.help = Some("do it in a statement before the call".into());
                }
            }
        }
    }
}

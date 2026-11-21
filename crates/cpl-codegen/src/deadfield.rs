//! Struct fields nothing reads, and the locals that only fill them.
//!
//! `Paths` keeps a `back` witness array and a settle `order` for whoever
//! asks for a path; a program that only reads distances never does. A field
//! is dropped when no code in the program reads it: no `.field`, no write
//! into it, and the struct's value is never printed, formatted, debugged,
//! compared, hashed, sorted, read or handed to a judge. A dropped field is a
//! `bool` placeholder in the C++. A local whose only uses are pure writes
//! (`x[pos] = v`, `x.push(v)`, `x.clear()`) and being stored into a dropped
//! field is dropped with it. Nothing dropped can trap or print, so every
//! build behaves exactly as the interpreter does.

use std::collections::BTreeSet;

use cpl_ir::{
    Arg, AssignOp, Block, DimSpec, Expr, ExprKind, FmtPiece, Mutation, Place, PlaceKind, Program,
    ScopeId, Slot, Stmt, Ty,
};

use cpl_ir::walk;

#[derive(Default)]
pub(crate) struct DeadFields {
    fields: BTreeSet<(usize, usize)>,
    locals: BTreeSet<(ScopeId, Slot)>,
    /// Kept locals read only where a dropped write or field reads them
    /// without effect: the C++ marks them used there.
    unheard: BTreeSet<(ScopeId, Slot)>,
}

impl DeadFields {
    pub fn collect(program: &Program) -> DeadFields {
        if std::env::var_os("CPL_KEEP_FIELDS").is_some() {
            return DeadFields::default();
        }
        let mut live = BTreeSet::new();
        let mut whole = BTreeSet::new();
        for s in program.structs.iter().enumerate() {
            if s.1.eq_fn.is_some() || s.1.lt_fn.is_some() || s.1.show_fn.is_some() {
                whole.insert(s.0);
            }
        }
        for &id in program.entry_fns.iter().chain(&program.judge_fns) {
            let f = &program.fns[id];
            structs_in(&f.ret, &mut whole);
            for local in f.locals.iter().take(f.params.len()) {
                structs_in(&local.ty, &mut whole);
            }
        }
        let mut tys: Vec<&Ty> = program.globals.iter().map(|g| &g.ty).collect();
        tys.extend(program.main_locals.iter().map(|l| &l.ty));
        for f in &program.fns {
            tys.extend(f.locals.iter().map(|l| &l.ty));
            tys.push(&f.ret);
        }
        for c in &program.closures {
            tys.extend(c.locals.iter().map(|l| &l.ty));
        }
        for s in &program.structs {
            tys.extend(s.fields.iter().map(|(_, t)| t));
        }
        for t in tys {
            keys_in(t, &mut whole);
        }
        for body in bodies(program) {
            reads(body.1, &mut live, &mut whole);
        }
        let mut work: Vec<usize> = whole.iter().copied().collect();
        while let Some(s) = work.pop() {
            for (_, t) in &program.structs[s].fields {
                let mut inner = BTreeSet::new();
                structs_in(t, &mut inner);
                for i in inner {
                    if whole.insert(i) {
                        work.push(i);
                    }
                }
            }
        }
        let mut dead = DeadFields::default();
        for (s, def) in program.structs.iter().enumerate() {
            if whole.contains(&s) {
                continue;
            }
            for (i, (_, t)) in def.fields.iter().enumerate() {
                if !live.contains(&(s, i)) && !matches!(t, Ty::Fn(..) | Ty::Unit) {
                    dead.fields.insert((s, i));
                }
            }
        }
        if dead.fields.is_empty() {
            return dead;
        }
        for (scope, body, locals, fixed) in bodies(program) {
            dead.locals_of(program, scope, body, locals, fixed);
        }
        for (scope, body, ..) in bodies(program) {
            dead.unheard_in(program, scope, body);
        }
        dead
    }

    pub fn unheard(&self, scope: ScopeId, slot: Slot) -> bool {
        self.unheard.contains(&(scope, slot))
    }

    fn unheard_in(&mut self, program: &Program, scope: ScopeId, body: &Block) {
        let mut quiet: BTreeSet<*const Expr> = BTreeSet::new();
        let mark = |e: &Expr, quiet: &mut BTreeSet<*const Expr>| {
            walk::deep_expr(e, &mut |x| {
                if let ExprKind::Local(_) = x.kind {
                    quiet.insert(x as *const Expr);
                }
            })
        };
        walk::deep_stmts(body, &mut |stmt| {
            let dropped = match stmt {
                Stmt::Let(slot, _) => self.local(scope, *slot),
                Stmt::Assign(place, ..) => self.place(scope, place),
                Stmt::Expr(Expr { kind: ExprKind::Mutate(_, place, _), .. }) => self.place(scope, place),
                _ => false,
            };
            if dropped {
                for e in parts(stmt).into_iter().filter(|e| pure(e)) {
                    mark(e, &mut quiet);
                }
            }
        });
        walk::deep_exprs(body, &mut |e| {
            if let (ExprKind::Tuple(items), Ty::Struct(s)) = (&e.kind, &e.ty) {
                for (i, item) in items.iter().enumerate() {
                    if self.field(*s, i) && pure(item) {
                        mark(item, &mut quiet);
                    }
                }
            }
        });
        if quiet.is_empty() {
            return;
        }
        let mut heard = BTreeSet::new();
        let mut only_quiet = BTreeSet::new();
        walk::deep_exprs(body, &mut |e| match e.kind {
            ExprKind::Local(slot) if quiet.contains(&(e as *const Expr)) => {
                only_quiet.insert(slot);
            }
            ExprKind::Local(slot) => {
                heard.insert(slot);
            }
            ExprKind::Closure(cid) => {
                heard.extend(program.closures[cid].captures.iter().filter(|c| !c.is_global).map(|c| c.outer));
            }
            _ => {}
        });
        for slot in only_quiet {
            if !heard.contains(&slot) && !self.local(scope, slot) {
                self.unheard.insert((scope, slot));
            }
        }
    }

    pub fn field(&self, s: usize, i: usize) -> bool {
        self.fields.contains(&(s, i))
    }

    pub fn local(&self, scope: ScopeId, slot: Slot) -> bool {
        self.locals.contains(&(scope, slot))
    }

    /// The place's variable is a dropped local.
    pub fn place(&self, scope: ScopeId, place: &Place) -> bool {
        let (global, slot) = walk::root(place);
        !global && self.local(scope, slot)
    }


    fn locals_of(&mut self, program: &Program, scope: ScopeId, body: &Block, n: usize, fixed: BTreeSet<Slot>) {
        let mut stored = BTreeSet::new();
        let mut written = BTreeSet::new();
        let mut read = fixed;
        walk::deep_exprs(body, &mut |e| {
            if let (ExprKind::Tuple(items), Ty::Struct(s)) = (&e.kind, &e.ty) {
                for (i, item) in items.iter().enumerate() {
                    if self.field(*s, i) && matches!(item.kind, ExprKind::Local(_)) {
                        stored.insert(item as *const Expr);
                    }
                }
            }
            if let ExprKind::Closure(cid) = e.kind {
                read.extend(program.closures[cid].captures.iter().filter(|c| !c.is_global).map(|c| c.outer));
            }
        });
        walk::deep_stmts(body, &mut |stmt| match stmt {
            Stmt::Let(..) => {}
            Stmt::Assign(place, op, _) => {
                if matches!(op, AssignOp::Set | AssignOp::Max | AssignOp::Min) && plain(place) {
                    written.insert(walk::root(place).1);
                } else {
                    read.insert(walk::root(place).1);
                }
            }
            Stmt::Expr(e) if quiet_mutate(e) => {
                written.insert(walk::root(mutated(e)).1);
            }
            Stmt::For(_, slot, ..) => {
                read.insert(*slot);
            }
            _ => {}
        });
        let mut quiet = BTreeSet::new();
        walk::deep_stmts(body, &mut |stmt| {
            if let Stmt::Expr(e) = stmt {
                if quiet_mutate(e) {
                    quiet.insert(e as *const Expr);
                }
            }
        });
        walk::deep_exprs(body, &mut |e| match &e.kind {
            ExprKind::Local(slot) if !stored.contains(&(e as *const Expr)) => {
                read.insert(*slot);
            }
            ExprKind::Mutate(_, place, _) if !quiet.contains(&(e as *const Expr)) => {
                read.insert(walk::root(place).1);
            }
            ExprKind::Sort(place, ..) => {
                read.insert(walk::root(place).1);
            }
            ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                for arg in args {
                    if let Arg::Mut(place) = arg {
                        read.insert(walk::root(place).1);
                    }
                }
            }
            _ => {}
        });
        for slot in 0..n {
            if !read.contains(&slot) && (written.contains(&slot) || stored_slot(&stored, slot)) {
                self.locals.insert((scope, slot));
            }
        }
    }
}

fn stored_slot(stored: &BTreeSet<*const Expr>, slot: Slot) -> bool {
    // SAFETY: the pointers are to expressions of the program, which outlives this pass.
    stored.iter().any(|&e| matches!(unsafe { &*e }.kind, ExprKind::Local(s) if s == slot))
}

/// Every body of code: its scope, block, local count, and the slots it
/// never owns (parameters and closure captures).
fn bodies(program: &Program) -> Vec<(ScopeId, &Block, usize, BTreeSet<Slot>)> {
    let mut out = vec![
        (ScopeId::Main, &program.setup, program.main_locals.len(), BTreeSet::new()),
        (ScopeId::Main, &program.main, program.main_locals.len(), BTreeSet::new()),
    ];
    for (id, f) in program.fns.iter().enumerate() {
        out.push((ScopeId::Fn(id), &f.body, f.locals.len(), (0..f.params.len()).collect()));
    }
    for (id, c) in program.closures.iter().enumerate() {
        let mut own: BTreeSet<Slot> = c.params.iter().chain(&c.muts).copied().collect();
        own.extend(c.captures.iter().map(|cap| cap.inner));
        out.push((ScopeId::Closure(id), &c.body, c.locals.len(), own));
    }
    out
}

/// The fields `body` reads, and the structs it uses whole.
fn reads(body: &Block, live: &mut BTreeSet<(usize, usize)>, whole: &mut BTreeSet<usize>) {
    walk::deep_exprs(body, &mut |e| match &e.kind {
        ExprKind::Field(base, i) => {
            if let Ty::Struct(s) = strip(&base.ty) {
                live.insert((*s, *i));
            }
        }
        ExprKind::Cmp(_, a, b) => {
            structs_in(&a.ty, whole);
            structs_in(&b.ty, whole);
        }
        ExprKind::Print(args, _) => args.iter().for_each(|a| structs_in(&a.value.ty, whole)),
        ExprKind::Debug(_, args) => args.iter().for_each(|a| structs_in(&a.value.ty, whole)),
        ExprKind::Format(pieces) => {
            for piece in pieces {
                if let FmtPiece::Value(a) = piece {
                    structs_in(&a.value.ty, whole);
                }
            }
        }
        ExprKind::Read(_) => structs_in(&e.ty, whole),
        ExprKind::Builtin(_, args) => args.iter().for_each(|a| structs_in(&a.ty, whole)),
        ExprKind::Sort(place, ..) => structs_in(&place.ty, whole),
        ExprKind::Mutate(how, place, _) => {
            if !matches!(how, Mutation::Push | Mutation::Clear | Mutation::Reserve) {
                structs_in(&place.ty, whole);
            }
            field_places(place, live);
        }
        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
            for arg in args {
                if let Arg::Mut(place) = arg {
                    field_places(place, live);
                }
            }
        }
        _ => {}
    });
    walk::deep_stmts(body, &mut |stmt| {
        if let Stmt::Assign(place, ..) = stmt {
            field_places(place, live);
        }
    });
}

fn field_places(place: &Place, live: &mut BTreeSet<(usize, usize)>) {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => {}
        PlaceKind::Field(base, i) => {
            if let Ty::Struct(s) = strip(&base.ty) {
                live.insert((*s, *i));
            }
            field_places(base, live);
        }
        PlaceKind::Index(base, _) | PlaceKind::Some(base) => field_places(base, live),
    }
}

fn strip(t: &Ty) -> &Ty {
    match t {
        Ty::Mut(t) => strip(t),
        t => t,
    }
}

fn structs_in(t: &Ty, out: &mut BTreeSet<usize>) {
    match t {
        Ty::Struct(s) => {
            out.insert(*s);
        }
        Ty::Range(t) | Ty::Opt(t) | Ty::List(t) | Ty::ListView(t) | Ty::Set(t) | Ty::HashSet(t) | Ty::Mut(t) => {
            structs_in(t, out)
        }
        Ty::Array { elem, .. } => structs_in(elem, out),
        Ty::Map(k, v) | Ty::HashMap(k, v) => {
            structs_in(k, out);
            structs_in(v, out);
        }
        Ty::Fn(args, ret) => {
            args.iter().for_each(|a| structs_in(a, out));
            structs_in(ret, out);
        }
        Ty::Tuple(ts) => ts.iter().for_each(|t| structs_in(t, out)),
        Ty::Record(fs) => fs.iter().for_each(|(_, t)| structs_in(t, out)),
        Ty::Enum(_) => {}
        _ => {}
    }
}

/// Structs inside the keys of maps and sets, which are hashed or ordered whole.
fn keys_in(t: &Ty, out: &mut BTreeSet<usize>) {
    match t {
        Ty::Set(k) | Ty::HashSet(k) => structs_in(k, out),
        Ty::Map(k, v) | Ty::HashMap(k, v) => {
            structs_in(k, out);
            keys_in(v, out);
        }
        Ty::Range(t) | Ty::Opt(t) | Ty::List(t) | Ty::ListView(t) | Ty::Mut(t) => keys_in(t, out),
        Ty::Array { elem, .. } => keys_in(elem, out),
        Ty::Tuple(ts) => ts.iter().for_each(|t| keys_in(t, out)),
        Ty::Record(fs) => fs.iter().for_each(|(_, t)| keys_in(t, out)),
        _ => {}
    }
}

/// Runs no code that can trap, print or change anything.
pub(crate) fn pure(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Unit
        | ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::None
        | ExprKind::Local(_)
        | ExprKind::Global(_)
        | ExprKind::DomainLen(_)
        | ExprKind::DomainLo(_)
        | ExprKind::DomainVal(_) => true,
        ExprKind::Some(a) | ExprKind::Field(a, _) => pure(a),
        ExprKind::Tuple(items) | ExprKind::Variant(_, items) => items.iter().all(pure),
        _ => false,
    }
}

/// What a dropped write to a dropped local still has to run: every
/// expression of it that could trap or print.
pub(crate) fn effects(stmt: &Stmt) -> Vec<&Expr> {
    let mut out = parts(stmt);
    out.retain(|e| !pure(e));
    out
}

/// The locals a dropped write or field value reads without effect, so the
/// C++ can still mark them used.
pub(crate) fn quiet_reads(stmt: Option<&Stmt>, item: Option<&Expr>) -> Vec<Slot> {
    let mut quiet: Vec<&Expr> = stmt.map(parts).unwrap_or_default();
    quiet.extend(item);
    quiet.retain(|e| pure(e));
    let mut out = Vec::new();
    for e in quiet {
        cpl_ir::walk::deep_expr(e, &mut |x| {
            if let ExprKind::Local(slot) = x.kind {
                if !out.contains(&slot) {
                    out.push(slot);
                }
            }
        });
    }
    out
}

fn parts(stmt: &Stmt) -> Vec<&Expr> {
    let mut out: Vec<&Expr> = Vec::new();
    match stmt {
        Stmt::Let(_, Some(init)) => match &init.kind {
            ExprKind::Fill(dims, value) => {
                for d in dims {
                    if let DimSpec::Int(len) | DimSpec::IntFrom(_, len) = d {
                        out.push(len);
                    }
                }
                out.extend(value.as_deref());
            }
            _ => out.push(init),
        },
        Stmt::Assign(place, _, value) => {
            index_exprs(place, &mut out);
            out.push(value);
        }
        Stmt::Expr(e) => {
            if let ExprKind::Mutate(_, place, args) = &e.kind {
                index_exprs(place, &mut out);
                out.extend(args);
            }
        }
        _ => {}
    }
    out
}

fn index_exprs<'e>(place: &'e Place, out: &mut Vec<&'e Expr>) {
    match &place.kind {
        PlaceKind::Index(base, idxs) => {
            index_exprs(base, out);
            out.extend(idxs);
        }
        PlaceKind::Field(base, _) | PlaceKind::Some(base) => index_exprs(base, out),
        PlaceKind::Local(_) | PlaceKind::Global(_) => {}
    }
}

fn quiet_mutate(e: &Expr) -> bool {
    matches!(&e.kind, ExprKind::Mutate(Mutation::Push | Mutation::Clear, place, _) if plain(place))
}

fn mutated(e: &Expr) -> &Place {
    match &e.kind {
        ExprKind::Mutate(_, place, _) => place,
        _ => unreachable!(),
    }
}

/// A variable, or cells of an array at positions of its domain: a write
/// there cannot trap once its indices are made.
fn plain(place: &Place) -> bool {
    match &place.kind {
        PlaceKind::Local(_) => true,
        PlaceKind::Index(base, idxs) => {
            matches!(strip(&base.ty), Ty::Array { .. })
                && idxs.iter().all(|i| matches!(i.ty, Ty::Pos(_)))
                && plain(base)
        }
        _ => false,
    }
}

/// A struct field the C++ does not store, for `cpl explain`.
pub struct DroppedField {
    /// `struct Paths.back`.
    pub site: String,
    /// The locals that only filled it, each as `name in fn`.
    pub fills: Vec<String>,
}

pub fn explain_dead_fields(program: &Program) -> Vec<DroppedField> {
    let dead = DeadFields::collect(program);
    let scope_name = |scope: ScopeId| match scope {
        ScopeId::Main => "main".to_string(),
        ScopeId::Fn(id) => format!("fn {}", program.fns[id].name),
        ScopeId::Closure(id) => format!("closure {id}"),
    };
    let local_name = |scope: ScopeId, slot: Slot| match scope {
        ScopeId::Main => &program.main_locals[slot].name,
        ScopeId::Fn(id) => &program.fns[id].locals[slot].name,
        ScopeId::Closure(id) => &program.closures[id].locals[slot].name,
    };
    let fills: Vec<String> = dead
        .locals
        .iter()
        .map(|&(scope, slot)| format!("{} in {}", local_name(scope, slot), scope_name(scope)))
        .collect();
    let cpp = crate::emit(program, "explain", None);
    dead.fields
        .iter()
        .filter(|&&(s, _)| cpp.contains(&format!("struct {} {{", crate::names::ident(&program.structs[s].name))))
        .map(|&(s, i)| {
            let def = &program.structs[s];
            DroppedField { site: format!("struct {}.{}", def.name, def.fields[i].0), fills: fills.clone() }
        })
        .collect()
}

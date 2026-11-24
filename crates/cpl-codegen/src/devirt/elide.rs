//! Elision: fn params, struct fields and locals nothing reads, and which slots may drop their fn value.

use super::*;

impl<'p> Devirt<'p> {
    pub(super) fn compute_elidable(&mut self) {
        let mut assigned_fields: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut captured_params: BTreeSet<(FnId, usize)> = BTreeSet::new();
        let mut scan_assigns = |home: Option<FnId>, block: &Block| {
            walk::deep_stmts(block, &mut |stmt| {
                if let Stmt::Assign(place, _, _) = stmt {
                    if let (PlaceKind::Field(base, index), Ty::Struct(s)) =
                        (&place.kind, place_base_ty(place))
                    {
                        let _ = base;
                        assigned_fields.insert((*s, *index));
                    }
                    if let (PlaceKind::Local(slot), Some(id)) = (&place.kind, home) {
                        if *slot < self.program.fns[id].params.len() {
                            self.reassigned_params.insert((id, *slot));
                        }
                    }
                }
            });
            walk::deep_exprs(block, &mut |e| {
                if let (ExprKind::Closure(cid), Some(id)) = (&e.kind, home) {
                    for cap in &self.program.closures[*cid].captures {
                        if !cap.is_global && cap.outer < self.program.fns[id].params.len() {
                            captured_params.insert((id, cap.outer));
                        }
                    }
                }
                if let ExprKind::Call(_, args) = &e.kind {
                    for arg in args {
                        if let Arg::Mut(place) = arg {
                            if let (PlaceKind::Field(_, index), Ty::Struct(s)) =
                                (&place.kind, place_base_ty(place))
                            {
                                assigned_fields.insert((*s, *index));
                            }
                            if let (PlaceKind::Local(slot), Some(id)) = (&place.kind, home) {
                                if *slot < self.program.fns[id].params.len() {
                                    self.reassigned_params.insert((id, *slot));
                                }
                            }
                        }
                    }
                }
            });
        };
        scan_assigns(None, &self.program.setup);
        scan_assigns(None, &self.program.main);
        for (id, f) in self.program.fns.iter().enumerate() {
            scan_assigns(Some(id), &f.body);
        }
        for closure in &self.program.closures {
            scan_assigns(None, &closure.body);
        }

        for (&home, &held) in &self.held {
            if !matches!(held, Held::One(_)) {
                continue;
            }
            match home {
                Home::Field(s, idx) => {
                    if !assigned_fields.contains(&(s, idx)) {
                        self.elidable.insert(home);
                    }
                }
                Home::Param(id, slot) => {
                    let f = &self.program.fns[id];
                    let forbidden = self.program.entry_fns.contains(&id)
                        || self.program.judge_fns.contains(&id)
                        || is_hook_fn(self.program, id)
                        || f.params[slot].mutable
                        || self.reassigned_params.contains(&(id, slot))
                        || captured_params.contains(&(id, slot));
                    if !forbidden {
                        self.elidable.insert(home);
                    }
                }
            }
        }

        // Fixed-point shrink: remove any home read in an escaping position.
        loop {
            let mut removed = Vec::new();
            self.find_escapes(None, &self.program.setup, &mut removed);
            self.find_escapes(None, &self.program.main, &mut removed);
            for (id, f) in self.program.fns.iter().enumerate() {
                self.find_escapes(Some(id), &f.body, &mut removed);
            }
            for closure in &self.program.closures {
                self.find_escapes(None, &closure.body, &mut removed);
            }
            let before = self.elidable.len();
            for h in removed {
                self.elidable.remove(&h);
            }
            if self.elidable.len() == before {
                break;
            }
        }
        self.elide_unread_fn_params();
        // A dropped field leaves locals dead, and a dead `let f = s.op`
        // reads `op` no more.
        loop {
            let before = (self.unread_fields.len(), self.dead_locals.len());
            self.drop_unread_fields();
            self.drop_dead_locals();
            if (self.unread_fields.len(), self.dead_locals.len()) == before {
                break;
            }
        }
    }

    /// A fn-valued parameter the body never names (`Heap.new`'s `by:` when
    /// it was specialized for `None`) holds nothing anyone calls. When every
    /// call passes it a value that runs nothing, it is elided like a known
    /// closure: a `bool` placeholder, and callers pass `false`.
    pub(super) fn elide_unread_fn_params(&mut self) {
        let program = self.program;
        let mut kept: BTreeSet<(FnId, usize)> = BTreeSet::new();
        let mut scan = |block: &Block| {
            walk::deep_exprs(block, &mut |e| {
                if let ExprKind::Call(id, args) = &e.kind {
                    for (slot, arg) in args.iter().enumerate() {
                        if !matches!(arg, Arg::Value(v) if self.skippable_value(v)) {
                            kept.insert((*id, slot));
                        }
                    }
                }
            });
        };
        scan(&program.setup);
        scan(&program.main);
        program.fns.iter().for_each(|f| scan(&f.body));
        program.closures.iter().for_each(|c| scan(&c.body));
        for (id, f) in program.fns.iter().enumerate() {
            if program.entry_fns.contains(&id)
                || program.judge_fns.contains(&id)
                || is_hook_fn(program, id)
            {
                continue;
            }
            let named = crate::scope::named_params(&f.body, f.params.len());
            let mut captured = BTreeSet::new();
            walk::deep_exprs(&f.body, &mut |e| {
                if let ExprKind::Closure(cid) = &e.kind {
                    captured.extend(
                        program.closures[*cid]
                            .captures
                            .iter()
                            .filter(|c| !c.is_global)
                            .map(|c| c.outer),
                    );
                }
            });
            for (slot, param) in f.params.iter().enumerate() {
                let fn_valued = match &f.locals[slot].ty {
                    Ty::Fn(..) => true,
                    Ty::Opt(inner) => matches!(**inner, Ty::Fn(..)),
                    _ => false,
                };
                if fn_valued
                    && !param.mutable
                    && !named.contains(&slot)
                    && !captured.contains(&slot)
                    && !kept.contains(&(id, slot))
                {
                    self.elidable.insert(Home::Param(id, slot));
                }
            }
        }
    }

    /// A field no expression reads is left out of its struct, and every
    /// store to it with it (decision 178). Each store must be a plain write
    /// of a value that runs nothing and cannot trap, so leaving it out
    /// changes nothing. A data field of a struct whose whole value is
    /// observed (compared, hashed, shown, read from input) is kept, since
    /// that reads every field.
    pub(super) fn drop_unread_fields(&mut self) {
        let program = self.program;
        let mut read: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut unskippable: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut whole: BTreeSet<usize> = BTreeSet::new();
        // `home` is the fn or main whose dead `let`s read nothing; `None`
        // for a closure's body.
        let mut scan = |home: Option<Option<FnId>>, block: &Block| {
            let mut gone: BTreeSet<*const Expr> = BTreeSet::new();
            walk::deep_stmts(block, &mut |stmt| match stmt {
                Stmt::Assign(place, op, _) if !self.skippable_store(place, *op) => {
                    place_fields(place, &mut unskippable);
                }
                Stmt::Let(slot, Some(init))
                    if home.is_some_and(|home| self.dead_local(home, *slot)) =>
                {
                    walk::deep_expr(init, &mut |e| {
                        gone.insert(e as *const Expr);
                    });
                }
                _ => {}
            });
            walk::deep_exprs(block, &mut |e| {
                match &e.kind {
                    ExprKind::Field(base, idx) if !gone.contains(&(e as *const Expr)) => {
                        if let Ty::Struct(s) = base.ty {
                            read.insert((s, *idx));
                        }
                    }
                    ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                        for arg in args {
                            if let Arg::Mut(place) = arg {
                                place_fields(place, &mut unskippable);
                            }
                        }
                    }
                    ExprKind::Tuple(items) => {
                        if let Ty::Struct(s) = e.ty {
                            for (idx, item) in items.iter().enumerate() {
                                if !self.skippable_value(item) {
                                    unskippable.insert((s, idx));
                                }
                            }
                        }
                    }
                    ExprKind::MutSlice(place, ..) | ExprKind::Mutate(_, place, _) => {
                        place_fields(place, &mut unskippable);
                        key_structs(&place.ty, &mut whole);
                    }
                    ExprKind::Sort(place, by, _) => {
                        place_fields(place, &mut unskippable);
                        if matches!(by, cpl_ir::SortBy::Natural) {
                            ty_structs(&place.ty, &mut whole);
                        }
                    }
                    ExprKind::Read(_) => ty_structs(&e.ty, &mut whole),
                    _ => {}
                }
                key_structs(&e.ty, &mut whole);
                if !keeps_fields(e) {
                    walk::children(e, &mut |child| ty_structs(&child.ty, &mut whole));
                }
            });
        };
        scan(Some(None), &program.setup);
        scan(Some(None), &program.main);
        for (id, f) in program.fns.iter().enumerate() {
            scan(Some(Some(id)), &f.body);
        }
        for closure in &program.closures {
            scan(None, &closure.body);
        }
        let locals = program
            .fns
            .iter()
            .flat_map(|f| &f.locals)
            .chain(program.closures.iter().flat_map(|c| &c.locals))
            .chain(&program.main_locals);
        for l in locals {
            key_structs(&l.ty, &mut whole);
        }
        for def in &program.structs {
            for (_, ty) in &def.fields {
                key_structs(ty, &mut whole);
            }
        }
        loop {
            let before = whole.len();
            for sid in whole.clone() {
                for (_, ty) in &program.structs[sid].fields {
                    ty_structs(ty, &mut whole);
                }
            }
            if whole.len() == before {
                break;
            }
        }
        for (sid, def) in program.structs.iter().enumerate() {
            let niche = crate::names::struct_niche(program, sid);
            for (idx, (name, ty)) in def.fields.iter().enumerate() {
                let key = (sid, idx);
                if read.contains(&key)
                    || unskippable.contains(&key)
                    || (!is_fn_or_opt_fn(ty)
                        && (whole.contains(&sid) || niche == Some(name.as_str())))
                {
                    continue;
                }
                self.elidable.insert(Home::Field(sid, idx));
                self.unread_fields.insert(key);
            }
        }
    }

    /// A local whose only uses are stores of plain values and being placed
    /// in a dropped field is dropped too, with those stores.
    pub(super) fn drop_dead_locals(&mut self) {
        let program = self.program;
        for (fid, f) in program.fns.iter().enumerate() {
            let params: Vec<bool> = f.params.iter().map(|p| p.mutable).collect();
            while self.find_dead_locals(Some(fid), &params, &[&f.body]) {}
        }
        while self.find_dead_locals(None, &[], &[&program.setup, &program.main]) {}
    }

    /// One round: a store or push left out reads nothing, so what it alone
    /// used can go in the next round.
    pub(super) fn find_dead_locals(
        &mut self,
        home: Option<FnId>,
        params: &[bool],
        blocks: &[&Block],
    ) -> bool {
        let mut lets: BTreeMap<usize, usize> = BTreeMap::new();
        let mut live: BTreeSet<usize> = BTreeSet::new();
        let mut feeds: BTreeSet<*const Expr> = BTreeSet::new();
        let mut pushes: BTreeSet<*const Expr> = BTreeSet::new();
        let mut dropped: BTreeSet<*const Expr> = BTreeSet::new();
        let mut effectful: BTreeSet<usize> = BTreeSet::new();
        let mut written: BTreeSet<usize> = BTreeSet::new();
        let dead = |s: &Self, slot: usize| s.dead_locals.contains(&(home, slot));
        for block in blocks {
            walk::deep_stmts(block, &mut |stmt| match stmt {
                Stmt::Expr(e) => {
                    if let Some(slot) = self.skippable_push(e) {
                        written.insert(slot);
                        pushes.insert(e as *const Expr);
                        if dead(self, slot) {
                            walk::deep_expr(e, &mut |inner| {
                                dropped.insert(inner as *const Expr);
                            });
                        }
                    }
                }
                Stmt::Let(slot, init) => {
                    *lets.entry(*slot).or_default() += 1;
                    if let Some(e) = init.as_ref().filter(|e| !self.skippable_value(e)) {
                        if matches!(e.kind, ExprKind::Closure(_)) {
                            live.insert(*slot);
                        } else {
                            effectful.insert(*slot);
                        }
                    }
                }
                Stmt::Assign(place, op, value) => {
                    let root = walk::root(place);
                    if let (false, slot) = root {
                        written.insert(slot);
                    }
                    if let (false, slot) = root {
                        if !self.skippable_store(place, *op) {
                            live.insert(slot);
                        }
                    }
                    let dead_root = matches!(root, (false, slot) if dead(self, slot));
                    if (dead_root || self.in_unread_field(place))
                        && self.skippable_store(place, *op)
                    {
                        let mut skip = |e: &Expr| {
                            walk::deep_expr(e, &mut |inner| {
                                dropped.insert(inner as *const Expr);
                            })
                        };
                        walk::place_exprs(place, &mut |e| skip(e));
                        if self.skippable_value(value) {
                            skip(value);
                        }
                    }
                }
                _ => {}
            });
            walk::deep_exprs(block, &mut |e| {
                if let ExprKind::CallExpr(callee, _) = &e.kind {
                    if skippable(callee) && self.target(home, callee).is_some() {
                        // The call is direct, so no part of the path is read.
                        let mut path: &Expr = callee;
                        feeds.insert(path as *const Expr);
                        while let ExprKind::Field(base, _) = &path.kind {
                            path = base;
                            feeds.insert(path as *const Expr);
                        }
                    }
                }
                if let (ExprKind::Tuple(items), Ty::Struct(s)) = (&e.kind, &e.ty) {
                    for (idx, item) in items.iter().enumerate() {
                        if self.unread_fields.contains(&(*s, idx))
                            && matches!(item.kind, ExprKind::Local(_))
                        {
                            feeds.insert(item as *const Expr);
                        }
                    }
                }
            });
        }
        let place_live = |place: &cpl_ir::Place, live: &mut BTreeSet<usize>| {
            if let (false, slot) = walk::root(place) {
                live.insert(slot);
            }
        };
        for block in blocks {
            walk::deep_exprs(block, &mut |e| {
                if dropped.contains(&(e as *const Expr)) {
                    return;
                }
                match &e.kind {
                    ExprKind::Local(slot) if !feeds.contains(&(e as *const Expr)) => {
                        live.insert(*slot);
                    }
                    ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                        for arg in args {
                            if let Arg::Mut(place) = arg {
                                place_live(place, &mut live);
                            }
                        }
                    }
                    ExprKind::Mutate(..) if pushes.contains(&(e as *const Expr)) => {}
                    ExprKind::MutSlice(place, ..)
                    | ExprKind::Mutate(_, place, _)
                    | ExprKind::Sort(place, ..) => place_live(place, &mut live),
                    ExprKind::Closure(cid) => {
                        for c in &self.program.closures[*cid].captures {
                            if !c.is_global {
                                live.insert(c.outer);
                            }
                        }
                    }
                    _ => {}
                }
            });
        }
        let mut found = false;
        let candidates = lets
            .into_iter()
            .filter(|&(slot, n)| n == 1 && slot >= params.len())
            .map(|(slot, _)| slot)
            .chain((0..params.len()).filter(|&slot| !params[slot]));
        for slot in candidates.collect::<Vec<_>>() {
            if live.contains(&slot) {
                continue;
            }
            if !effectful.contains(&slot) {
                found |= self.dead_locals.insert((home, slot));
            } else if slot >= params.len() && !written.contains(&slot) {
                self.unread_lets.insert((home, slot));
            }
        }
        found
    }
}

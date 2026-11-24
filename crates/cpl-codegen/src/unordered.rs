//! Which `Map`s and `Set`s can be hash tables in the C++.
//!
//! A `Map<K, V>` is ordered, but a program that only stores, looks up and
//! removes never sees the order. For such a key type the C++ keeps a
//! hash table (salted, so no test can be built against it). The rule is
//! per key type over the whole program, since two maps of one type can
//! be assigned to each other. Anything that is not on the short list of
//! uses that can't see the order keeps the tree.

use crate::uses::effects::Runs;
use crate::walk;
use cpl_ir::{
    Block, Builtin, Expr, ExprKind, Iter, Mutation, PlaceKind, Program, SortBy, Stmt, Ty, UnOp,
};

#[derive(Default)]
struct Found {
    maps: Vec<Ty>,
    sets: Vec<Ty>,
    ordered_maps: Vec<Ty>,
    ordered_sets: Vec<Ty>,
    /// Loops over a table whose order cannot show: see [`sorted_scans`].
    blind_scans: Vec<*const Expr>,
}

fn add(list: &mut Vec<Ty>, ty: &Ty) {
    if !list.contains(ty) {
        list.push(ty.clone());
    }
}

/// Calls `f` on every `Map` and `Set` type inside `ty`, itself included.
fn tables(program: &Program, ty: &Ty, seen: &mut Vec<usize>, f: &mut dyn FnMut(&Ty)) {
    match ty {
        Ty::Map(key, value) => {
            f(ty);
            tables(program, key, seen, f);
            tables(program, value, seen, f);
        }
        Ty::Set(elem) => {
            f(ty);
            tables(program, elem, seen, f);
        }
        Ty::HashMap(key, value) => {
            tables(program, key, seen, f);
            tables(program, value, seen, f);
        }
        Ty::Opt(inner)
        | Ty::List(inner)
        | Ty::ListView(inner, _)
        | Ty::Keyed(inner, _)
        | Ty::HashSet(inner)
        | Ty::Range(inner) => tables(program, inner, seen, f),
        Ty::Array { elem, .. } => tables(program, elem, seen, f),
        Ty::Tuple(items) => items.iter().for_each(|t| tables(program, t, seen, f)),
        Ty::Struct(id) if !seen.contains(id) => {
            seen.push(*id);
            for (_, field) in &program.structs[*id].fields {
                tables(program, field, seen, f);
            }
        }
        _ => {}
    }
}

impl Found {
    fn see(&mut self, program: &Program, ty: &Ty) {
        tables(program, ty, &mut Vec::new(), &mut |t| match t {
            Ty::Map(key, _) => add(&mut self.maps, key),
            Ty::Set(elem) => add(&mut self.sets, elem),
            _ => {}
        });
    }

    /// The order of every table inside `ty` may show.
    fn order(&mut self, program: &Program, ty: &Ty) {
        tables(program, ty, &mut Vec::new(), &mut |t| match t {
            Ty::Map(key, _) => add(&mut self.ordered_maps, key),
            Ty::Set(elem) => add(&mut self.ordered_sets, elem),
            _ => {}
        });
    }

    fn expr(&mut self, program: &Program, e: &Expr) {
        self.see(program, &e.ty);
        // What only stores, finds, removes or passes a table along.
        let blind = match &e.kind {
            ExprKind::Local(_)
            | ExprKind::Global(_)
            | ExprKind::Field(..)
            | ExprKind::Index(..)
            | ExprKind::Tuple(_)
            | ExprKind::Variant(..)
            | ExprKind::Payload(..)
            | ExprKind::IsVariant(..)
            | ExprKind::Some(_)
            | ExprKind::IsSome(_)
            | ExprKind::Unwrap(..)
            | ExprKind::If(..)
            | ExprKind::Do(_)
            | ExprKind::Call(..)
            | ExprKind::CallExpr(..)
            | ExprKind::Closure(_) => true,
            ExprKind::Builtin(
                Builtin::Contains | Builtin::MapGet | Builtin::Len | Builtin::Copy,
                _,
            ) => true,
            ExprKind::Mutate(op, place, _) => {
                self.see(program, &place.ty);
                let blind = matches!(
                    op,
                    Mutation::Insert
                        | Mutation::Remove
                        | Mutation::Clear
                        | Mutation::Take
                        | Mutation::Push
                );
                if !blind {
                    self.order(program, &place.ty);
                }
                true
            }
            _ => false,
        };
        if !blind {
            self.order(program, &e.ty);
            walk::children(e, &mut |child| self.order(program, &child.ty));
        }
    }

    fn block(&mut self, program: &Program, block: &Block) {
        walk::deep_exprs(block, &mut |e| self.expr(program, e));
        walk::deep_stmts(block, &mut |stmt| {
            if let Stmt::For(_, _, iter, _) = stmt {
                match iter {
                    Iter::Walk { .. } => unreachable!("Iter::Walk is expanded before codegen"),
                    Iter::Seq(seq, _) if self.blind_scans.contains(&(seq as *const Expr)) => {}
                    Iter::Seq(seq, _)
                    | Iter::RangeVal(seq, _)
                    | Iter::TreeRange { container: seq, .. } => self.order(program, &seq.ty),
                    Iter::Range { .. } => {}
                }
            }
        });
    }
}

/// `[x for .. in m ..].sorted(..)` where equal sort keys mean equal
/// elements: a scalar element sorted by itself or its negation. The list
/// comes out the same in any order of `m`, so its loop cannot show it, as
/// long as the loop does nothing but push onto that list.
fn sorted_scans(block: &Block, out: &mut Vec<*const Expr>) {
    walk::deep_exprs(block, &mut |e| {
        let ExprKind::Do(b) = &e.kind else { return };
        // a sort may bind its arguments first: plain scalar
        // lets, which run nothing and can't see the list's order.
        let stmts: Vec<&Stmt> = b.stmts.iter().filter(|s| !simple_scalar_let(s)).collect();
        let [Stmt::Let(list, Some(init)), Stmt::Expr(sort), Stmt::Give(give)] = stmts[..] else {
            return;
        };
        let ExprKind::Sort(place, by, _) = &held_sort(sort).kind else {
            return;
        };
        let by_value = match by {
            SortBy::Natural => true,
            SortBy::Key(slot, key) => match &key.kind {
                ExprKind::Local(s) => s == slot,
                ExprKind::Unary(UnOp::Neg, x) => matches!(x.kind, ExprKind::Local(s) if s == *slot),
                _ => false,
            },
            SortBy::Fn { .. } => false,
        };
        let scalar = matches!(&place.ty, Ty::List(elem) if elem.is_scalar());
        if !by_value
            || !scalar
            || !matches!(place.kind, PlaceKind::Local(s) if s == *list)
            || !matches!(give.kind, ExprKind::Local(s) if s == *list)
        {
            return;
        }
        let built = match &init.kind {
            ExprKind::Builtin(Builtin::Copy, args) if args.len() == 1 => &args[0],
            _ => init,
        };
        let ExprKind::Do(inner) = &built.kind else {
            return;
        };
        let [Stmt::Let(acc, Some(empty)), Stmt::For(_, _, Iter::Seq(seq, _), body), Stmt::Give(out_list)] =
            &inner.stmts[..]
        else {
            return;
        };
        let table = matches!(seq.ty, Ty::Map(..) | Ty::Set(_));
        let fresh = matches!(&empty.kind, ExprKind::Tuple(items) if items.is_empty());
        if table
            && fresh
            && matches!(out_list.kind, ExprKind::Local(s) if s == *acc)
            && pushes_only_onto(body, *acc)
        {
            out.push(seq as *const Expr);
        }
    });
}

/// The sort in `sort`, or in `do: let d = desc; sort; ()` that
/// a sort with bound arguments lowers to.
fn held_sort(sort: &Expr) -> &Expr {
    let ExprKind::Do(b) = &sort.kind else {
        return sort;
    };
    let rest: Vec<&Stmt> = b.stmts.iter().filter(|s| !simple_scalar_let(s)).collect();
    match rest[..] {
        [Stmt::Expr(inner), Stmt::Give(unit)] if unit.kind == ExprKind::Unit => inner,
        _ => sort,
    }
}

/// `let x = v` of a scalar literal or name: runs nothing.
fn simple_scalar_let(s: &Stmt) -> bool {
    matches!(s, Stmt::Let(_, Some(v)) if v.ty.is_scalar() && cpl_ir::walk::is_simple(v))
}

/// `body` changes nothing but pushes onto the local `acc`, calls nothing
/// and never leaves early.
fn pushes_only_onto(body: &Block, acc: usize) -> bool {
    let runs = Runs::of_block(body);
    let mut ok = !runs.code() && !runs.prints;
    walk::deep_exprs(body, &mut |e| match &e.kind {
        ExprKind::Mutate(Mutation::Push, place, _) => {
            ok &= matches!(place.kind, PlaceKind::Local(s) if s == acc);
        }
        ExprKind::Mutate(..) => ok = false,
        _ => {}
    });
    walk::deep_stmts(body, &mut |stmt| {
        ok &= !matches!(stmt, Stmt::Break(_) | Stmt::Return(_) | Stmt::Assign(..));
    });
    ok
}

/// The key types of the `Map`s, and the element types of the `Sets`,
/// whose order the program can never see.
pub(crate) fn find(program: &Program) -> (Vec<Ty>, Vec<Ty>) {
    let mut found = Found::default();
    let mut bodies: Vec<&Block> = program.fns.iter().map(|f| &f.body).collect();
    bodies.extend(program.closures.iter().map(|c| &c.body));
    bodies.extend([&program.main, &program.setup]);
    for body in bodies {
        sorted_scans(body, &mut found.blind_scans);
    }
    for global in &program.globals {
        found.see(program, &global.ty);
    }
    for f in &program.fns {
        found.block(program, &f.body);
    }
    for c in &program.closures {
        found.block(program, &c.body);
    }
    found.block(program, &program.main);
    found.block(program, &program.setup);
    let Found {
        mut maps,
        mut sets,
        ordered_maps,
        ordered_sets,
        ..
    } = found;
    maps.retain(|k| !ordered_maps.contains(k));
    sets.retain(|k| !ordered_sets.contains(k));
    (maps, sets)
}

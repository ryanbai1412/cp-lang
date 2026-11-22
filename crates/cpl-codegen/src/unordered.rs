//! Which `Map`s and `Set`s can be hash tables in the C++.
//!
//! A `Map<K, V>` is ordered, but a program that only stores, looks up and
//! removes never sees the order. For such a key type the C++ keeps a
//! hash table (salted, so no test can be built against it). The rule is
//! per key type over the whole program, since two maps of one type can
//! be assigned to each other. Anything that is not on the short list of
//! uses that can't see the order keeps the tree.

use crate::walk;
use cpl_ir::{Block, Builtin, Expr, ExprKind, Iter, Mutation, Program, Stmt, Ty};

#[derive(Default)]
struct Found {
    maps: Vec<Ty>,
    sets: Vec<Ty>,
    ordered_maps: Vec<Ty>,
    ordered_sets: Vec<Ty>,
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
                    Iter::Seq(seq, _)
                    | Iter::RangeVal(seq, _)
                    | Iter::Subsets(seq)
                    | Iter::Members(seq)
                    | Iter::TreeRange { container: seq, .. } => self.order(program, &seq.ty),
                    Iter::Range { .. } => {}
                }
            }
        });
    }
}

/// The key types of the `Map`s, and the element types of the `Sets`,
/// whose order the program can never see.
pub(crate) fn find(program: &Program) -> (Vec<Ty>, Vec<Ty>) {
    let mut found = Found::default();
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
    } = found;
    maps.retain(|k| !ordered_maps.contains(k));
    sets.retain(|k| !ordered_sets.contains(k));
    (maps, sets)
}

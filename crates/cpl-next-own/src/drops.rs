//! Where a value is released (R1). Ownership places every release and the runtimes only run
//! them: a frame releases `Func::drops` when the fn leaves, and `Stmt::Drop` releases a
//! discarded value and the value an overwrite replaces.

use cpl_next_ir::ir::{addressed, visit_places, Expr, Place, Stmt, EK};
use cpl_next_ir::prog::{plain, CTy, Program};

pub(crate) fn place(p: &mut Program) {
    let mut insts = std::mem::take(&mut p.insts);
    for f in &mut insts {
        let plain = |t: &CTy| plain(&|s| p.parts(s), t, 0);
        let lent = |l: usize| f.params.iter().any(|&(x, lent)| x == l && lent);
        let owned: Vec<bool> = (f.locals.iter().enumerate())
            .map(|(l, t)| !lent(l) && holds(&plain, t))
            .collect();
        f.drops = (0..owned.len()).filter(|&l| owned[l]).collect();
        // A call may read or write a local whose address is taken, so the local's old value
        // is released only once the new one is computed.
        let calm: Vec<bool> = (0..owned.len())
            .map(|l| owned[l] && !addressed(&f.body, l))
            .collect();
        let mut body = std::mem::take(&mut f.body);
        Placer {
            locals: &mut f.locals,
            plain: &plain,
            owned: &owned,
            calm: &calm,
        }
        .block(&mut body);
        f.body = body;
    }
    p.insts = insts;
}

/// Whether a value of type `t` may hold storage it must release; a lent slot points at a
/// value another frame owns.
fn holds(plain: &dyn Fn(&CTy) -> bool, t: &CTy) -> bool {
    !matches!(t, CTy::Lent(_)) && !plain(t)
}

struct Placer<'a> {
    locals: &'a mut Vec<CTy>,
    plain: &'a dyn Fn(&CTy) -> bool,
    /// The locals the frame releases; a temporary made here is released where it is made.
    owned: &'a [bool],
    /// The owned locals nothing but the fn itself reads or writes.
    calm: &'a [bool],
}

impl Placer<'_> {
    fn block(&mut self, b: &mut Vec<Stmt<CTy>>) {
        for st in std::mem::take(b) {
            self.stmt(st, b);
        }
    }
    fn stmt(&mut self, mut st: Stmt<CTy>, out: &mut Vec<Stmt<CTy>>) {
        match &mut st {
            Stmt::If(_, a, b) | Stmt::Loop(a, b, _) => {
                self.block(a);
                self.block(b);
            }
            Stmt::Switch(_, arms) => arms.iter_mut().for_each(|a| self.block(a)),
            Stmt::Region { body, .. } => self.block(body),
            _ => {}
        }
        match st {
            Stmt::Expr(e) if holds(self.plain, &e.ty) => {
                let t = self.temp(&e.ty);
                out.push(Stmt::Let(t, e));
                out.push(Stmt::Drop(Place::Local(t)));
            }
            Stmt::Let(l, e) if self.owned.get(l) == Some(&true) => {
                self.overwrite(Place::Local(l), e, true, out)
            }
            Stmt::Assign(p, e) if holds(self.plain, &e.ty) => self.overwrite(p, e, false, out),
            st => out.push(st),
        }
    }
    /// `p = e` releases what `p` held. It does so first when computing `e` cannot read `p`;
    /// else `e` goes to a temporary first, which also keeps the order in which `e` and an
    /// index of `p` trap.
    fn overwrite(&mut self, p: Place, e: Expr<CTy>, local: bool, out: &mut Vec<Stmt<CTy>>) {
        let set = |p: Place, e| match p {
            Place::Local(l) if local => Stmt::Let(l, e),
            p => Stmt::Assign(p, e),
        };
        if let Place::Local(l) = p {
            if self.calm.get(l) == Some(&true) && !reads(&e, l) {
                out.push(Stmt::Drop(p.clone()));
                out.push(set(p, e));
                return;
            }
        }
        let t = self.temp(&e.ty);
        let copy = Expr {
            kind: EK::Copy(Place::Local(t)),
            ty: e.ty.clone(),
            site: 0,
        };
        out.push(Stmt::Let(t, e));
        out.push(Stmt::Drop(p.clone()));
        out.push(set(p, copy));
        out.push(Stmt::Drop(Place::Local(t)));
    }
    fn temp(&mut self, t: &CTy) -> usize {
        self.locals.push(t.clone());
        self.locals.len() - 1
    }
}

/// Whether `e` names local `l` in any place it reads, writes or lends.
fn reads(e: &Expr<CTy>, l: usize) -> bool {
    let mut named = false;
    visit_places(&mut [Stmt::Expr(e.clone())], &mut |p| {
        named |= matches!(p.root_mut(), Place::Local(x) if *x == l);
    });
    named
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copy(l: usize) -> Expr<CTy> {
        Expr {
            kind: EK::Copy(Place::Local(l)),
            ty: CTy::Raw(Box::new(CTy::Int)),
            site: 0,
        }
    }
    fn shape(s: &Stmt<CTy>) -> String {
        let at = |p: &Place| match p {
            Place::Local(l) => l.to_string(),
            _ => "field".into(),
        };
        let of = |e: &Expr<CTy>| match &e.kind {
            EK::Copy(p) => at(p),
            _ => "?".into(),
        };
        match s {
            Stmt::Let(l, e) => format!("{l} = {}", of(e)),
            Stmt::Assign(p, e) => format!("{} = {}", at(p), of(e)),
            Stmt::Drop(p) => format!("drop {}", at(p)),
            Stmt::Expr(e) => of(e),
            _ => "?".into(),
        }
    }

    /// A discarded value is dropped in a temporary; an overwrite drops the old value first,
    /// unless the new value reads it or the place is more than a local.
    #[test]
    fn every_release_is_a_drop_where_it_happens() {
        let heap = CTy::Raw(Box::new(CTy::Int));
        let mut locals = vec![heap.clone(), heap];
        let mut body = vec![
            Stmt::Expr(copy(1)),
            Stmt::Let(0, copy(1)),
            Stmt::Assign(Place::Local(0), copy(0)),
            Stmt::Assign(Place::Field(Box::new(Place::Local(1)), 0), copy(0)),
        ];
        Placer {
            locals: &mut locals,
            plain: &|t| matches!(t, CTy::Int),
            owned: &[true, true],
            calm: &[true, true],
        }
        .block(&mut body);
        let got: Vec<String> = body.iter().map(shape).collect();
        let want = [
            "2 = 1",
            "drop 2", //
            "drop 0",
            "0 = 1", //
            "3 = 0",
            "drop 0",
            "0 = 3",
            "drop 3", //
            "4 = 0",
            "drop field",
            "field = 4",
            "drop 4",
        ];
        assert_eq!(got, want);
        assert_eq!(locals.len(), 5);
    }
}

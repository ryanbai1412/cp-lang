//! How a value of each type lies in words. A type's layout is a number; `Layouts` gives its
//! size, its shape and whether it holds storage handles, which copying and dropping count.
use cpl_next_ir::prog::{CTy, Program};
use std::collections::HashMap;

pub(crate) type Lay = u32;

#[derive(Clone, Debug)]
pub(crate) enum Shape {
    /// No words: `Unit`.
    Zero,
    /// One word: an integer, a Bool, a Char, a U64, a domain position or a stack address.
    Word,
    /// One word naming a heap object (0: none yet).
    Handle,
    /// Fields at their word offsets.
    Struct(Vec<(u32, Lay)>),
    /// A tag word, then the payload of the member it names.
    Union(Vec<Lay>),
}

#[derive(Default)]
pub(crate) struct Layouts {
    pub shape: Vec<Shape>,
    pub size: Vec<u32>,
    /// The offsets of the handles a value holds, when they are the same for every value (no
    /// union decides them).
    pub flat: Vec<Option<Vec<u32>>>,
    of: HashMap<CTy, Lay>,
}

impl Layouts {
    pub fn of(&mut self, p: &Program, t: &CTy) -> Lay {
        if let Some(&l) = self.of.get(t) {
            return l;
        }
        let rep = p.rep(t);
        if rep != t {
            let l = self.of(p, rep);
            self.of.insert(t.clone(), l);
            return l;
        }
        let (shape, size) = match t {
            CTy::Unit | CTy::Error => (Shape::Zero, 0),
            CTy::Int
            | CTy::Range(..)
            | CTy::Bool
            | CTy::Char
            | CTy::U64
            | CTy::F64
            | CTy::Dom(_)
            | CTy::Ref(_) => (Shape::Word, 1),
            // Two words, low then high.
            CTy::I128 => {
                let w = self.of(p, &CTy::Int);
                (Shape::Struct(vec![(0, w), (1, w)]), 2)
            }
            CTy::Raw(_) => (Shape::Handle, 1),
            // A lent value's words are its owner's: the same shape, holding nothing to release.
            CTy::Lent(e) => {
                let l = self.of(p, e) as usize;
                let at = self.push(t, self.shape[l].clone(), self.size[l]);
                self.flat[at as usize] = Some(Vec::new());
                return at;
            }
            CTy::Fn(_, _, ms) => {
                let ls: Vec<Lay> = ms
                    .iter()
                    .map(|k| self.of(p, &CTy::Clo(Box::new(k.clone()))))
                    .collect();
                self.union(ls)
            }
            CTy::Struct(..) if p.sums.contains_key(t) => {
                let ls = p.sums[t].iter().map(|m| self.of(p, m)).collect();
                self.union(ls)
            }
            CTy::Struct(..) => self.fields(p, &p.parts(t)),
            CTy::Clo(k) => self.fields(p, &p.captures(k)),
        };
        let l = self.push(t, shape, size);
        debug_assert!(
            p.stores(t) || self.flat[l as usize].as_deref() == Some(&[]),
            "{t:?} holds a handle that `Program::stores` misses"
        );
        l
    }
    fn push(&mut self, t: &CTy, shape: Shape, size: u32) -> Lay {
        let l = self.shape.len() as Lay;
        let flat = match &shape {
            Shape::Zero | Shape::Word => Some(Vec::new()),
            Shape::Handle => Some(vec![0]),
            Shape::Struct(fs) => fs.iter().try_fold(Vec::new(), |mut v, (at, f)| {
                v.extend(self.flat[*f as usize].as_ref()?.iter().map(|o| at + o));
                Some(v)
            }),
            // A union's handles depend on its member, unless no member holds one.
            Shape::Union(ls) => ls
                .iter()
                .all(|m| self.flat[*m as usize].as_deref() == Some(&[]))
                .then(Vec::new),
        };
        self.flat.push(flat);
        self.shape.push(shape);
        self.size.push(size);
        self.of.insert(t.clone(), l);
        l
    }
    fn fields(&mut self, p: &Program, ts: &[CTy]) -> (Shape, u32) {
        let mut at = 0;
        let mut fs = Vec::with_capacity(ts.len());
        for t in ts {
            let l = self.of(p, t);
            fs.push((at, l));
            at += self.size[l as usize];
        }
        (Shape::Struct(fs), at)
    }
    fn union(&mut self, ls: Vec<Lay>) -> (Shape, u32) {
        let size = 1 + ls.iter().map(|l| self.size[*l as usize]).max().unwrap_or(0);
        (Shape::Union(ls), size)
    }
    /// The word offset and layout of field `i` of a struct, or of member `i`'s payload.
    pub fn part(&self, l: Lay, i: usize) -> Option<(u32, Lay)> {
        match &self.shape[l as usize] {
            Shape::Struct(fs) => fs.get(i).copied(),
            Shape::Union(ls) => ls.get(i).map(|l| (1, *l)),
            _ => None,
        }
    }
    /// The handles a value of layout `l` in `words` holds.
    pub fn handles(&self, l: Lay, words: &[u64], out: &mut Vec<u32>) {
        if let Some(at) = &self.flat[l as usize] {
            let hs = at.iter().map(|o| words[*o as usize]).filter(|&h| h != 0);
            out.extend(hs.map(|h| h as u32));
            return;
        }
        match &self.shape[l as usize] {
            Shape::Zero | Shape::Word => {}
            Shape::Handle => {
                if words[0] != 0 {
                    out.push(words[0] as u32);
                }
            }
            Shape::Struct(fs) => {
                for (at, f) in fs {
                    self.handles(*f, &words[*at as usize..], out);
                }
            }
            Shape::Union(ls) => {
                if let Some(m) = ls.get(words[0] as usize) {
                    self.handles(*m, &words[1..], out);
                }
            }
        }
    }
}

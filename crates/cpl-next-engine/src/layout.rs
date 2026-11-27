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
    /// Whether a value holds a handle, so copying or dropping it touches counts.
    pub owns: Vec<bool>,
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
        let (shape, size, owns) = match t {
            CTy::Unit | CTy::Error => (Shape::Zero, 0, false),
            CTy::Int
            | CTy::Range(..)
            | CTy::Bool
            | CTy::Char
            | CTy::U64
            | CTy::F64
            | CTy::Dom(_)
            | CTy::Ref(_) => (Shape::Word, 1, false),
            CTy::Raw(_) => (Shape::Handle, 1, true),
            CTy::Lent(t) => {
                let l = self.of(p, t);
                self.of.insert(CTy::Lent(t.clone()), l);
                return l;
            }
            CTy::Set(ms) => {
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
        self.push(t, shape, size, owns)
    }
    fn push(&mut self, t: &CTy, shape: Shape, size: u32, owns: bool) -> Lay {
        let l = self.shape.len() as Lay;
        let flat = match &shape {
            Shape::Zero | Shape::Word => Some(Vec::new()),
            Shape::Handle => Some(vec![0]),
            Shape::Struct(fs) => fs.iter().try_fold(Vec::new(), |mut v, (at, f)| {
                v.extend(self.flat[*f as usize].as_ref()?.iter().map(|o| at + o));
                Some(v)
            }),
            Shape::Union(_) => None,
        };
        self.flat.push(flat);
        self.shape.push(shape);
        self.size.push(size);
        self.owns.push(owns);
        self.of.insert(t.clone(), l);
        l
    }
    fn fields(&mut self, p: &Program, ts: &[CTy]) -> (Shape, u32, bool) {
        let mut at = 0;
        let mut fs = Vec::with_capacity(ts.len());
        for t in ts {
            let l = self.of(p, t);
            fs.push((at, l));
            at += self.size[l as usize];
        }
        let owns = fs.iter().any(|(_, l)| self.owns[*l as usize]);
        (Shape::Struct(fs), at, owns)
    }
    fn union(&mut self, ls: Vec<Lay>) -> (Shape, u32, bool) {
        let size = 1 + ls.iter().map(|l| self.size[*l as usize]).max().unwrap_or(0);
        let owns = ls.iter().any(|l| self.owns[*l as usize]);
        (Shape::Union(ls), size, owns)
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
        if !self.owns[l as usize] {
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

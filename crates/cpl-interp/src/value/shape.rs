//! The shapes [`super::Cells::Rows`] packs: tuples of tuples over scalars and
//! small enums, optional or not, such as `?(from: V, via: (to: V, w: Int))`,
//! each value a row of words the way the C++ stores the struct. A shape is
//! its nodes in pre-order, a nibble each: a scalar is its [`SmallFieldKind`],
//! a tuple is [`TUP`] then its field count, an enum is [`ENUM`] (four words,
//! as [`try_unbox_small_enum`] packs it). An optional shape starts with
//! [`OPT`] and its row with a word saying whether it is there (all zero when
//! not).

use super::cells::{box_small_enum, try_unbox_small_enum};
use super::{OptVal, Shared, SmallFieldKind, TupleVal, Value};

const TUP: u64 = 4;
const OPT: u64 = 5;
const ENUM: u64 = 6;
/// The most nodes a shape has, leaving the top bit for [`UNORDERED`].
const NODES: usize = 15;
/// The most words a row takes.
const MAX_WORDS: usize = 16;
/// Set on a shape with an enum, whose words do not keep the values' order.
pub(crate) const UNORDERED: u64 = 1 << 63;

fn nib(shape: u64, s: usize) -> u64 {
    (shape >> (4 * s)) & 15
}

struct Build {
    shape: u64,
    nodes: usize,
    words: usize,
}

impl Build {
    fn node(&mut self, node: u64) -> Option<()> {
        (self.nodes < NODES).then(|| {
            self.shape |= node << (4 * self.nodes);
            self.nodes += 1;
        })
    }
}

/// The shape of `v` and the words of its row, when it packs.
pub(crate) fn of(v: &Value) -> Option<(u64, u8)> {
    let mut b = Build {
        shape: 0,
        nodes: 0,
        words: 0,
    };
    match v {
        Value::Opt(OptVal::Boxed(x)) => {
            b.node(OPT)?;
            b.words = 1;
            walk(x, &mut b)?;
        }
        _ => walk(v, &mut b)?,
    }
    (1..=MAX_WORDS)
        .contains(&b.words)
        .then_some((b.shape, b.words as u8))
}

fn walk(v: &Value, b: &mut Build) -> Option<()> {
    if let Some((kind, _)) = SmallFieldKind::try_unbox(v) {
        b.words += 1;
        return b.node(kind as u64);
    }
    if let Value::Tuple(TupleVal::Boxed(fs)) = v {
        if fs.len() > 15 {
            return None;
        }
        b.node(TUP)?;
        b.node(fs.len() as u64)?;
        for f in fs.iter() {
            walk(f, b)?;
        }
        return (b.words <= MAX_WORDS).then_some(());
    }
    try_unbox_small_enum(v)?;
    b.words += 4;
    b.shape |= UNORDERED;
    b.node(ENUM)
}

/// Writes `v` into `row`; false when `v` is not of `shape`.
pub(crate) fn pack(shape: u64, v: &Value, row: &mut [i64]) -> bool {
    if nib(shape, 0) != OPT {
        return put(shape, &mut 0, v, row, &mut 0);
    }
    match v {
        Value::Opt(OptVal::None) => {
            row.fill(0);
            true
        }
        Value::Opt(OptVal::Boxed(x)) => {
            row[0] = 1;
            put(shape, &mut 1, x, row, &mut 1)
        }
        _ => false,
    }
}

fn put(shape: u64, s: &mut usize, v: &Value, row: &mut [i64], d: &mut usize) -> bool {
    let node = nib(shape, *s);
    *s += 1;
    if node == TUP {
        let n = nib(shape, *s) as usize;
        *s += 1;
        let Value::Tuple(TupleVal::Boxed(fs)) = v else {
            return false;
        };
        return n == fs.len() && fs.iter().all(|f| put(shape, s, f, row, d));
    }
    if node == ENUM {
        let Some(words) = try_unbox_small_enum(v) else {
            return false;
        };
        row[*d..*d + 4].copy_from_slice(&words);
        *d += 4;
        return true;
    }
    match SmallFieldKind::try_unbox(v) {
        Some((kind, x)) if kind as u64 == node => {
            row[*d] = x;
            *d += 1;
            true
        }
        _ => false,
    }
}

/// The value `row` holds.
pub(crate) fn unpack(shape: u64, row: &[i64]) -> Value {
    if nib(shape, 0) != OPT {
        return take(shape, &mut 0, row, &mut 0);
    }
    if row[0] == 0 {
        return Value::Opt(OptVal::None);
    }
    Value::Opt(OptVal::some(take(shape, &mut 1, row, &mut 1)))
}

fn take(shape: u64, s: &mut usize, row: &[i64], d: &mut usize) -> Value {
    let node = nib(shape, *s);
    *s += 1;
    if node == TUP {
        let n = nib(shape, *s) as usize;
        *s += 1;
        let fields = Shared::collect(n, |_| take(shape, s, row, d));
        return Value::Tuple(TupleVal::Boxed(fields));
    }
    if node == ENUM {
        *d += 4;
        return box_small_enum(&row[*d - 4..*d]);
    }
    *d += 1;
    SmallFieldKind::of_bits(node as i64).box_val(row[*d - 1])
}

/// A tuple shape made of `fields`, each a scalar's [`SmallFieldKind`] or a
/// tuple's shape with its node count; `None` past [`NODES`].
pub(crate) fn tuple(fields: &[(u64, usize)]) -> Option<(u64, usize)> {
    let mut b = Build {
        shape: TUP | (fields.len() as u64) << 4,
        nodes: 2,
        words: 0,
    };
    for &(shape, nodes) in fields {
        (b.nodes + nodes <= NODES && fields.len() < 16).then_some(())?;
        b.shape |= shape << (4 * b.nodes);
        b.nodes += nodes;
    }
    Some((b.shape, b.nodes))
}

/// Where field `f` of the scalar tuple `shape` starts in its words, and its
/// shape when it is a tuple too; `None` past the last field.
pub(crate) fn field(shape: u64, f: usize) -> Option<(usize, Option<u64>)> {
    (f < nib(shape, 1) as usize).then_some(())?;
    let (mut s, mut d) = (2, 0);
    for _ in 0..f {
        d += skip(shape, &mut s);
    }
    let start = s;
    skip(shape, &mut s);
    let sub = (shape >> (4 * start)) & ((1u64 << (4 * (s - start))) - 1);
    Some((d, (nib(sub, 0) == TUP).then_some(sub)))
}

/// Steps `s` past the node there and what it holds: the words it takes.
fn skip(shape: u64, s: &mut usize) -> usize {
    let node = nib(shape, *s);
    *s += 1;
    if node != TUP {
        return 1;
    }
    let n = nib(shape, *s);
    *s += 1;
    (0..n).map(|_| skip(shape, s)).sum()
}

/// Whether `shape` is a tuple of scalars, nested or not, a word each.
pub(crate) fn is_scalar_tuple(shape: u64) -> bool {
    nib(shape, 0) == TUP && shape & UNORDERED == 0
}

/// Writes the scalars of `v`, a tuple of tuples over scalars, into `out`.
pub(crate) fn leaves(v: &Value, out: &mut [i64], d: &mut usize) {
    match v {
        Value::Tuple(TupleVal::Boxed(fs)) => fs.iter().for_each(|f| leaves(f, out, d)),
        _ => {
            if let Some(o) = out.get_mut(*d) {
                *o = SmallFieldKind::try_unbox(v).map_or(0, |(_, x)| x);
            }
            *d += 1;
        }
    }
}

/// The field count of a tuple shape whose fields are a scalar word each.
pub(crate) fn flat_len(shape: u64) -> Option<usize> {
    let n = nib(shape, 1) as usize;
    (nib(shape, 0) == TUP && (0..n).all(|i| nib(shape, 2 + i) < TUP)).then_some(n)
}

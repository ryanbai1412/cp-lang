use lib::prelude::*;
use rand::Rng;
use std::ops::{Add, AddAssign};

type P<T> = Option<Box<T>>;

const IINF: i32 = 1e9 as i32;

#[derive(Debug, Copy, Clone)]
struct NodeValue {
    sum: i32,
    pref: i32,
    suff: i32,
    v: i32,
}

impl Default for NodeValue {
    fn default() -> Self {
        Self {
            sum: 0,
            pref: -IINF,
            suff: -IINF,
            v: -IINF,
        }
    }
}

impl Add<NodeValue> for NodeValue {
    type Output = NodeValue;

    fn add(self, rhs: NodeValue) -> Self::Output {
        NodeValue {
            sum: self.sum + rhs.sum,
            pref: self.pref.max(self.sum + rhs.pref),
            suff: rhs.suff.max(self.suff + rhs.sum),
            v: self.v.max(rhs.v).max(self.suff + rhs.pref),
        }
    }
}

impl NodeValue {
    fn new(x: i32) -> Self {
        Self {
            sum: x,
            pref: x,
            suff: x,
            v: x,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct Lazy {
    set: Option<i32>,
    rev: bool,
}

impl AddAssign<Lazy> for Lazy {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, rhs: Lazy) {
        if rhs.set.is_some() {
            *self = rhs;
        } else if rhs.rev {
            self.rev ^= true;
        }
    }
}

impl Lazy {
    fn apply(&self, n: &mut NodeValue, siz: i32) {
        if let Some(set) = self.set {
            n.sum = set * siz;
            let v = set.max(set * siz);
            n.pref = v;
            n.suff = v;
            n.v = v;
        } else if self.rev {
            swap(&mut n.pref, &mut n.suff);
        }
    }

    fn new_set(x: i32) -> Self {
        Self {
            set: Some(x),
            rev: false,
        }
    }

    fn new_rev() -> Self {
        Self {
            set: None,
            rev: true,
        }
    }
}

struct Node {
    v: NodeValue,
    lazy: Lazy,
    x: i32,
    size: i32,
    pri: i32,
    lc: P<Node>,
    rc: P<Node>,
}

impl Node {
    fn new(x: i32, rng: &mut impl Rng) -> Self {
        Self {
            v: NodeValue::new(x),
            lazy: Default::default(),
            x,
            size: 1,
            pri: rng.gen(),
            lc: None,
            rc: None,
        }
    }

    fn size(n: Option<&Node>) -> i32 {
        n.map(|n| n.size).unwrap_or_default()
    }

    fn v(n: Option<&Node>) -> NodeValue {
        n.map(|n| n.v).unwrap_or_default()
    }
}

fn pushdown(o: &mut Node) {
    let go = |c: Option<&mut Node>| {
        let Some(c) = c else {
            return;
        };
        o.lazy.apply(&mut c.v, c.size);
        c.lazy += o.lazy;
    };
    go(o.lc.as_deref_mut());
    go(o.rc.as_deref_mut());
    if let Some(set) = o.lazy.set {
        o.x = set;
    }
    if o.lazy.rev {
        swap(&mut o.lc, &mut o.rc);
    }
    o.lazy = Default::default();
}

fn maintain(o: &mut Node) {
    o.size = Node::size(o.lc.as_deref()) + 1 + Node::size(o.rc.as_deref());
    o.v = Node::v(o.lc.as_deref()) + NodeValue::new(o.x) + Node::v(o.rc.as_deref());
    o.lazy.apply(&mut o.v, o.size);
}

fn split(o: P<Node>, ind: i32) -> (P<Node>, P<Node>) {
    let Some(mut o) = o else {
        return (None, None);
    };
    pushdown(&mut o);
    let lc_size = Node::size(o.lc.as_deref());
    if ind <= lc_size {
        let (l, r) = split(o.lc, ind);
        o.lc = r;
        maintain(&mut o);
        (l, Some(o))
    } else {
        let (l, r) = split(o.rc, ind - lc_size - 1);
        o.rc = l;
        maintain(&mut o);
        (Some(o), r)
    }
}

fn merge(l: P<Node>, r: P<Node>) -> P<Node> {
    let Some(mut l) = l else {
        return r;
    };
    let Some(mut r) = r else {
        return Some(l);
    };
    if l.pri < r.pri {
        pushdown(&mut l);
        l.rc = merge(l.rc, Some(r));
        maintain(&mut l);
        Some(l)
    } else {
        pushdown(&mut r);
        r.lc = merge(Some(l), r.lc);
        maintain(&mut r);
        Some(r)
    }
}

fn split3(o: P<Node>, ind: i32, len: i32) -> (P<Node>, P<Node>, P<Node>) {
    let (l, r) = split(o, ind);
    let (mid, r) = split(r, len);
    (l, mid, r)
}

fn merge3(l: P<Node>, m: P<Node>, r: P<Node>) -> P<Node> {
    merge(merge(l, m), r)
}

fn build(arr: Vec<i32>, rng: &mut impl Rng) -> P<Node> {
    fn build(arr: &[i32], pris: &[i32]) -> P<Node> {
        if arr.is_empty() {
            return None;
        }

        let mid = arr.len() / 2;
        let lc = build(&arr[..mid], &pris[..mid]);
        let rc = build(&arr[mid + 1..], &pris[mid + 1..]);
        Some(Box::new(Node {
            v: Node::v(lc.as_deref()) + NodeValue::new(arr[mid]) + Node::v(rc.as_deref()),
            lazy: Default::default(),
            x: arr[mid],
            size: Node::size(lc.as_deref()) + 1 + Node::size(rc.as_deref()),
            pri: pris[mid],
            lc,
            rc,
        }))
    }

    let pris = {
        let mut pris = (0..arr.len()).map(|_| rng.gen()).collect::<Vec<i32>>();
        pris.sort_unstable();
        pris
    };

    build(&arr, &pris)
}

#[derive(Default)]
struct Solver;

impl StandardSolver for Solver {
    fn solve<C: ReadExtConfig>(self, inp: &mut impl ReadExt<C>, out: &mut impl WriteExt) {
        let mut rng = rand::thread_rng();
        let (n, q) = inp.read::<(i32, i32)>();

        let mut root = build((0..n).map(|_| inp.read::<i32>()).collect(), &mut rng);

        for _ in 0..q {
            let s = inp.read::<String>();

            if s == b"MAX-SUM" {
                writeln!(out, "{}", root.as_ref().unwrap().v.v).unwrap();
                continue;
            }

            let (ind, len) = inp.read::<(i32, i32)>();

            if s == b"INSERT" {
                let (nl, nr) = split(root, ind);
                root = merge3(
                    nl,
                    build((0..len).map(|_| inp.read::<i32>()).collect(), &mut rng),
                    nr,
                );
                continue;
            }

            let ind = ind - 1;
            let (nl, mut nm, nr) = split3(root, ind, len);

            if s == b"DELETE" {
                root = merge(nl, nr);
                continue;
            }

            match &*s {
                b"MAKE-SAME" => {
                    let nm = nm.as_mut().unwrap();
                    nm.lazy += Lazy::new_set(inp.read::<i32>());
                    maintain(nm);
                }
                b"REVERSE" => {
                    let nm = nm.as_mut().unwrap();
                    nm.lazy += Lazy::new_rev();
                    maintain(nm);
                }
                b"GET-SUM" => {
                    writeln!(out, "{}", Node::v(nm.as_deref()).sum).unwrap();
                }
                _ => panic!("unknown query {}", String::from_utf8_lossy(&s)),
            }

            root = merge3(nl, nm, nr);
        }
    }
}

fn main() {
    solver::main::<Solver, io::config::NoOverflowChecks>(ProblemTestcaseType::SingleTest)
}
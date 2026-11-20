// ETSR '23 Round 3 P1 — Clamped Affine Function Composition (CP Lang v2 spec)
// original: corpus/submissions/5527625.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca) + stdlib/lazy.cp
//
// Apply a sequence of (+x, -x, *x, +x*orig) operations clamped to [ql, qr]
// after every step to `q` initial queries. Coordinate-compress the queries,
// track (lo, hi) endpoints per segment in `LazySegTree`, and clamp both ends
// in O(log q) with `max_right` / `min_left`.

struct Cell:
    lo_orig: Int
    lo_val: Int
    hi_orig: Int
    hi_val: Int

struct Op:
    set: ?Int = none
    mul: Int = 1
    m: Int = 0
    b: Int = 0

    fn eval(self, orig: Int, val: Int) -> Int:
        let base = self.set ?? val
        base * self.mul + self.m * orig + self.b

    fn compose(self, prev: Op) -> Op:
        if self.set != none: self
        else: Op(prev.set, prev.mul * self.mul, prev.m * self.mul + self.m, prev.b * self.mul + self.b)

let (n, ql, qr): (1..=1e5, Int, Int) = read()
domain Step = 0..n
let ops: [Step](t: 1..=4, x: Int) = read()
let q: 1..=1e5 = read()
domain Q = 0..q
let arr: [Q]Int = read()

let xs = compress(arr)
domain U = xs.D

let cell_m = Monoid<?Cell>(
    none,
    (a, b) =>
        if a == none: b
        elif b == none: a
        else: Cell(a!.lo_orig, a!.lo_val, b!.hi_orig, b!.hi_val),
)
let cell_act = Action<?Cell, Op>(
    Op(),
    (newer, older) => newer.compose(older),
    (op, c, _) =>
        if let x = c: Cell(x.lo_orig, op.eval(x.lo_orig, x.lo_val), x.hi_orig, op.eval(x.hi_orig, x.hi_val))
        else: none,
    over: cell_m,
)

var st = LazySegTree<U, ?Cell, Op>(cell_act, [U] u => Cell(xs[u], xs[u], xs[u], xs[u]))

for op in ops:
    let tag = match op.t:
        1 => Op(none, 1, 0, op.x)
        2 => Op(none, 1, 0, -op.x)
        3 => Op(none, op.x, 0, 0)
        4 => Op(none, 1, op.x, 0)
    st.apply(U.first..=U.last, tag)

    let left_end = st.max_right(U.first.l, c => if let x = c: x.hi_val < ql else: true)
    st.apply(U.first.l..left_end, Op(ql, 1, 0, 0))

    let right_start = st.min_left(U.last.r, c => if let x = c: x.lo_val > qr else: true)
    st.apply(right_start..U.last.r, Op(qr, 1, 0, 0))

for a in arr:
    print(st.get(xs.at(a))!.lo_val)

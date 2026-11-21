struct Seg:
    has: Bool
    val: Int
    tot: Int
    pref: Int
    suff: Int

fn empty_seg() -> Seg = Seg(false, 0, 0, 0, 0)

fn single_seg(v: Int) -> Seg = Seg(true, v, v, v, v)

fn concat(a: Seg, b: Seg) -> Seg:
    if not a.has: return b
    if not b.has: return a
    Seg(
        true,
        max(max(a.val, b.val), a.suff + b.pref),
        a.tot + b.tot,
        max(a.pref, a.tot + b.pref),
        max(b.suff, b.tot + a.suff),
    )

fn scale(a: Seg, s: Int) -> Seg = Seg(a.has, a.val * s, a.tot * s, a.pref * s, a.suff * s)

fn sgn(v: Int) -> Int = if v > 0: 1 elif v < 0: -1 else: 0

struct Op:
    has_set: Bool
    set_val: Int
    s0: Int
    s1: Int
    s2: Int
    p0: Seg
    p1: Seg
    p2: Seg
    tail: Seg

fn id_op() -> Op = Op(false, 0, -1, 0, 1, empty_seg(), empty_seg(), empty_seg(), empty_seg())

fn op_sign(o: Op, z: Int) -> Int = if z < 0: o.s0 elif z == 0: o.s1 else: o.s2

fn op_pref(o: Op, z: Int) -> Seg = if z < 0: o.p0 elif z == 0: o.p1 else: o.p2

struct Leaf:
    cur: Int
    hist: Seg

fn apply_leaf(b: Op, a: Leaf) -> Leaf:
    let z = sgn(a.cur)
    var hist = concat(a.hist, scale(op_pref(b, z), abs(a.cur)))
    var cur = abs(a.cur) * op_sign(b, z)
    if b.has_set:
        cur = b.set_val
        hist = concat(hist, b.tail)
    Leaf(cur, hist)

fn compose_op(b: Op, a: Op) -> Op:
    if a.has_set:
        let z = sgn(a.set_val)
        var tail = concat(a.tail, scale(op_pref(b, z), abs(a.set_val)))
        var sv = abs(a.set_val) * op_sign(b, z)
        if b.has_set:
            sv = b.set_val
            tail = concat(tail, b.tail)
        return Op(true, sv, a.s0, a.s1, a.s2, a.p0, a.p1, a.p2, tail)
    let (ns0, ns1, ns2) = (op_sign(b, a.s0), op_sign(b, a.s1), op_sign(b, a.s2))
    let (np0, np1, np2) = (
        concat(a.p0, op_pref(b, a.s0)),
        concat(a.p1, op_pref(b, a.s1)),
        concat(a.p2, op_pref(b, a.s2)),
    )
    if b.has_set: Op(true, b.set_val, ns0, ns1, ns2, np0, np1, np2, b.tail)
    else: Op(false, 0, ns0, ns1, ns2, np0, np1, np2, empty_seg())

@!multitest
let (n, q): (Int, Int) = read()
let init_v: [0..n]Int = read()
var leaves: [0..n]Leaf = Leaf(0, empty_seg())
for i in 0..n: leaves[i] = Leaf(init_v[i], single_seg(init_v[i]))
let act = Action(
    id: id_op(),
    compose: (b, a) => compose_op(b, a),
    apply: (f, v, span) => apply_leaf(f, v),
    over: Monoid(id: Leaf(0, empty_seg()), op: (a, b) => a),
)
var st = LazySegTree(act, leaves)
let adv = Op(false, 0, -1, 0, 1, single_seg(-1), single_seg(0), single_seg(1), empty_seg())
var last_ans: U64 = 0
for _ in 0..q:
    let op: Int = read()
    if op == 1:
        let (u1, u2): (U64, U64) = read()
        let x: Int = read()
        let (a, b) = (Int((u1 ^ last_ans) % U64(n)), Int((u2 ^ last_ans) % U64(n)))
        let cur = Op(true, x, -1, 0, 1, empty_seg(), empty_seg(), empty_seg(), empty_seg())
        st.apply(min(a, b)..=max(a, b), cur)
    elif op == 2:
        let (u1, u2): (U64, U64) = read()
        let (a, b) = (Int((u1 ^ last_ans) % U64(n)), Int((u2 ^ last_ans) % U64(n)))
        let cur = Op(false, 0, 1, 0, -1, empty_seg(), empty_seg(), empty_seg(), empty_seg())
        st.apply(min(a, b)..=max(a, b), cur)
    elif op == 3:
        let (u1, u2): (U64, U64) = read()
        let (a, b) = (Int((u1 ^ last_ans) % U64(n)), Int((u2 ^ last_ans) % U64(n)))
        let cur = Op(false, 0, 0, 0, 1, empty_seg(), empty_seg(), empty_seg(), empty_seg())
        st.apply(min(a, b)..=max(a, b), cur)
    else:
        let u: U64 = read()
        let idx = Int((u ^ last_ans) % U64(n))
        let ans = st.get(idx).hist.val
        print(ans)
        last_ans = U64(ans)
    st.apply(0..n, adv)

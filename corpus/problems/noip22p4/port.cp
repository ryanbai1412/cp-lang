// Source: NOIP 2022 P4 (Match), https://dmoj.ca/problem/noip22p4
// limits: 3000 ms, 1024 MB
// Problem: permutations a and b of 1..n and Q queries (l, r), n, Q <= 2.5e5 (the first line also
//   has a test id). For each query print the sum over all l <= p <= q <= r of
//   max(a[p..=q]) * max(b[p..=q]), mod 2^64 (test id T in 0..=25; T = 0 on samples).
// Solution: offline by right end r. A lazy SegTree over left ends p keeps A_p = max a[p..=r],
//   B_p = max b[p..=r] and the history sum H_p of A_p * B_p over all r so far. Monotone stacks
//   turn each new r into range assignments of A and of B; then every p <= r adds A_p * B_p to
//   H_p. Tags compose assignments with history coefficients (on AB, A, B and 1); a query with
//   right end r folds H over l..=r. All sums and products are U64, which wraps mod 2^64
//   exactly as the answer asks. O((n + Q) log n).
// original: corpus/submissions/5445685.cpp
// Bounds: test_id: 0..=25; n, q: 1..=250_000; a, b, qs.l, qs.r: 1..=n

let (test_id, n): (0..=25, 1..=250_000) = read()
domain I = 1..=n
let a: [I]1..=n = read()
let b: [I]1..=n = read()
let q: 1..=250_000 = read()
domain Q = 0..q
let qs: [Q](l: I, r: I) = read()

struct HistNode:
    len: U64
    sa: U64
    sb: U64
    sab: U64
    hist: U64

struct HistTag:
    has_a: Bool
    ta: U64
    has_b: Bool
    tb: U64
    cab: U64
    ca: U64
    cb: U64
    c1: U64

const Z = U64(0)
let id_tag = HistTag(false, Z, false, Z, Z, Z, Z, Z)

let hist_monoid = Monoid<HistNode>(
    id: HistNode(Z, Z, Z, Z, Z),
    op: (x, y) => HistNode(x.len + y.len, x.sa + y.sa, x.sb + y.sb, x.sab + y.sab, x.hist + y.hist),
)

// Compose `f` (newer) after `g` (older).
fn compose_tag(f: HistTag, g: HistTag) -> HistTag:
    var cab = g.cab
    var ca = g.ca
    var cb = g.cb
    var c1 = g.c1
    if g.has_a and g.has_b:
        c1 += f.cab * g.ta * g.tb + f.ca * g.ta + f.cb * g.tb + f.c1
    elif g.has_a:
        cb += f.cab * g.ta + f.cb
        c1 += f.ca * g.ta + f.c1
    elif g.has_b:
        ca += f.cab * g.tb + f.ca
        c1 += f.cb * g.tb + f.c1
    else:
        cab += f.cab
        ca += f.ca
        cb += f.cb
        c1 += f.c1
    let ha = f.has_a or g.has_a
    let ta = if f.has_a: f.ta else: g.ta
    let hb = f.has_b or g.has_b
    let tb = if f.has_b: f.tb else: g.tb
    HistTag(ha, ta, hb, tb, cab, ca, cb, c1)

fn apply_tag(f: HistTag, x: HistNode) -> HistNode:
    let nhist = x.hist + f.cab * x.sab + f.ca * x.sa + f.cb * x.sb + f.c1 * x.len
    var nsa = x.sa
    var nsb = x.sb
    var nsab = x.sab
    if f.has_a and f.has_b:
        nsa = f.ta * x.len
        nsb = f.tb * x.len
        nsab = f.ta * f.tb * x.len
    elif f.has_a:
        nsa = f.ta * x.len
        nsab = f.ta * x.sb
    elif f.has_b:
        nsb = f.tb * x.len
        nsab = f.tb * x.sa
    HistNode(x.len, nsa, nsb, nsab, nhist)

let hist_action = Action<HistNode, HistTag>(
    id: id_tag,
    compose: (f, g) => compose_tag(f, g),
    apply: (f, x, len) => apply_tag(f, x),
    over: hist_monoid,
)

var by_r: [I]List<Q> = [I] _ => []
for qi in Q:
    by_r[qs[qi].r].push(qi)

let init = [I] _ => HistNode(U64(1), Z, Z, Z, Z)
var st = SegTree(hist_action.over, init, act: hist_action)

var stk_a: List<Int> = []
var stk_b: List<Int> = []
var ans: [Q]U64 = [Q] _ => Z

for r in I:
    while stk_a.len > 0 and a[I.nth(stk_a.last!)] <= a[r]:
        stk_a.pop()
    let la = if stk_a.len == 0: 0 else: stk_a.last! + 1
    stk_a.push(r.ord)
    st.apply(I.nth(la)..=r, HistTag(true, U64(a[r]), false, Z, Z, Z, Z, Z))

    while stk_b.len > 0 and b[I.nth(stk_b.last!)] <= b[r]:
        stk_b.pop()
    let lb = if stk_b.len == 0: 0 else: stk_b.last! + 1
    stk_b.push(r.ord)
    st.apply(I.nth(lb)..=r, HistTag(false, Z, true, U64(b[r]), Z, Z, Z, Z))

    st.apply(..=r, HistTag(false, Z, false, Z, U64(1), Z, Z, Z))

    for qi in by_r[r]:
        ans[qi] = st.fold(qs[qi].l..=r).hist

for qi in Q: print(ans[qi])

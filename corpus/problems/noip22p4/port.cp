// Runnable port (proto/cp.py lib-run) of statement-first noip22p4.cp
// original: corpus/submissions/5445685.cpp
// Note: proto/cpc lacks U64 today, so Int (64-bit signed) is used; exact on all
// inputs where historical sum < 2^63 (see friction backlog for U64 wrapping gap).

let (test_id, n): (Int, Int) = read()
domain I = 1..=n
let a: [I]Int = read()
let b: [I]Int = read()
let q: Int = read()
domain Q = 0..q
let qs: [Q](l: I, r: I) = read()

struct HistNode:
    len: Int
    sa: Int
    sb: Int
    sab: Int
    hist: Int

struct HistTag:
    has_a: Bool
    ta: Int
    has_b: Bool
    tb: Int
    cab: Int
    ca: Int
    cb: Int
    c1: Int

let id_tag = HistTag(false, 0, false, 0, 0, 0, 0, 0)

let hist_monoid = Monoid<HistNode>(
    id: HistNode(0, 0, 0, 0, 0),
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

let init = [I] _ => HistNode(1, 0, 0, 0, 0)
var st = LazySegTree<I, HistNode, HistTag>(hist_action, init)

var stk_a: List<Int> = []
var stk_b: List<Int> = []
var ans: [Q]Int = [Q] _ => 0

for r in I:
    while stk_a.len > 0 and a[I.nth(stk_a.last!)] <= a[r]:
        stk_a.pop()
    let la = if stk_a.len == 0: 0 else: stk_a.last! + 1
    stk_a.push(r.to_idx)
    st.apply(I.nth(la)..=r, HistTag(true, a[r], false, 0, 0, 0, 0, 0))

    while stk_b.len > 0 and b[I.nth(stk_b.last!)] <= b[r]:
        stk_b.pop()
    let lb = if stk_b.len == 0: 0 else: stk_b.last! + 1
    stk_b.push(r.to_idx)
    st.apply(I.nth(lb)..=r, HistTag(false, 0, true, b[r], 0, 0, 0, 0))

    st.apply(..=r, HistTag(false, 0, false, 0, 1, 0, 0, 0))

    for qi in by_r[r]:
        ans[qi] = st.fold(qs[qi].l..=r).hist

for qi in Q: print(ans[qi])

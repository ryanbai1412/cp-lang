// Source: NOIP 2022 P4 (Match), https://dmoj.ca/problem/noip22p4
// limits: 3000 ms, 1024 MB
// Problem: permutations a and b of 1..n and Q queries (l, r), n, Q <= 2.5e5 (the first line also
//   has a test id). For each query print the sum over all l <= p <= q <= r of
//   max(a[p..=q]) * max(b[p..=q]), mod 2^64 (test id T in 0..=25; T = 0 on samples).
// Solution: offline by right end r. A lazy SegTree over left ends p keeps A_p = max a[p..=r],
//   B_p = max b[p..=r] and the history sum H_p of A_p * B_p over all r so far. a[r] is the max of
//   a[p..=r] exactly for p in its span (after the nearest earlier larger a), so each new r sets A
//   on that span, sets B on its own span, and adds A_p * B_p to H_p for every p <= r. A tag first
//   adds ab·AB + a·A + b·B + one to H, then sets A and B if given; a query with right end r folds
//   H over l..=r. All sums and products are U64, which wraps mod 2^64 exactly as the answer asks.
//   O((n + Q) log n).
// Bounds: _: 0..=25; n, q: 1..=250_000; a, b, qs.l, qs.r: 1..=n
// original: orig.cpp

let (_, n): (0..=25, 1..=250_000) = read()  // the test id is not used
domain I = 1..=n
let a: [I]1..=n = read()
let b: [I]1..=n = read()
let q: 1..=250_000 = read()
domain Query = 0..q
let qs: [Query](l: I, r: I) = read()

// Over a run of left ends: the sums of A, B, A·B and the history H.
struct Sums(a: U64, b: U64, ab: U64, hist: U64)
// Add ab·AB + a·A + b·B + one to every H, then set A and B where given.
struct Tag(set_a: ?U64, set_b: ?U64, ab: U64, a: U64, b: U64, one: U64)

// g, then f: f's history terms see the A and B that g set as constants.
fn then(f: Tag, g: Tag) -> Tag:
    var (ab, ka, kb, one) = (f.ab, f.a, f.b, f.one)
    if let x = g.set_a: (ab, ka, kb, one) = (U64(0), U64(0), kb + ab * x, one + ka * x)
    if let y = g.set_b: (ab, ka, kb, one) = (U64(0), ka + ab * y, U64(0), one + kb * y)
    Tag(f.set_a ?? g.set_a, f.set_b ?? g.set_b, g.ab + ab, g.a + ka, g.b + kb, g.one + one)

fn apply(f: Tag, s: Sums, len: Int) -> Sums:
    let w = U64(len)
    let ab = match (f.set_a, f.set_b):
        (Some(x), Some(y)) => x * y * w
        (Some(x), None) => x * s.b
        (None, Some(y)) => y * s.a
        (None, None) => s.ab
    Sums(
        f.set_a.map(x => x * w) ?? s.a,
        f.set_b.map(y => y * w) ?? s.b,
        ab,
        s.hist + f.ab * s.ab + f.a * s.a + f.b * s.b + f.one * w,
    )

let none = Sums(U64(0), U64(0), U64(0), U64(0))
let history = Action<Sums, Tag>(
    id: Tag(None, None, U64(0), U64(0), U64(0), U64(0)),
    compose: then,
    apply:,
    over: Monoid<Sums>(
        id: none,
        op: (x, y) => Sums(x.a + y.a, x.b + y.b, x.ab + y.ab, x.hist + y.hist),
    ),
)
let set_a = (x: U64) => Tag(Some(x), None, U64(0), U64(0), U64(0), U64(0))
let set_b = (y: U64) => Tag(None, Some(y), U64(0), U64(0), U64(0), U64(0))
let record = Tag(None, None, U64(1), U64(0), U64(0), U64(0))

let reign_a = a.spans(by: x => -x)
let reign_b = b.spans(by: x => -x)
let ending = ([Query] k => qs[k].r).buckets()
var st = SegTree(history.over, [I] _ => none, act: history)
var ans: [Query]U64 = U64(0)
for r in I:
    st.apply(reign_a[r].l..r.r, set_a(U64(a[r])))
    st.apply(reign_b[r].l..r.r, set_b(U64(b[r])))
    st.apply(..=r, record)
    for k in ending[r]: ans[k] = st.fold(qs[k].l..=r).hist
for k in Query: print(ans[k])

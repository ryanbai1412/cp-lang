// NOIP '22 P4 - Match / Contest (DMOJ: noip22p4, 25pt, TL 3.0s, ML 1024MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Two permutations a[1..n], b[1..n] (n <= 2.5e5) and q <= 2.5e5 queries (l, r).
// Compute sum_{l <= p <= q <= r} max(a[p..=q]) * max(b[p..=q]) mod 2^64.
//
// Offline right-endpoint sweep: advance r = 1..n, updating suffix ranges on two
// monotonic stacks for max(a) and max(b), then tick history += a[p] * b[p] on 1..=r
// in a 5-field / 6-tag Historical-Sum LazySegTree over U64 wrapping arithmetic.
use std.algebra.{Monoid, Action}
use std.lazy.{LazySegTree}

let _test_id: Int = read()
let n: 1..=250_000 = read()
domain I = 1..=n
// WISH A6: Permutation constraint (`a` and `b` are permutations of 1..=n) cannot
// be expressed directly in `read()` for `cpl stress` without `Perm<I>` input support.
let a: [I]U64 = read()
let b: [I]U64 = read()
let q: 1..=250_000 = read()
domain Q = 0..q
struct Query(l: I, r: I)
    where l <= r
let qs: [Q]Query = read()

// WISH A7: Heavy custom Monoid + Action boilerplate for historical-sum lazy trees.
// Defining a 5-field Node + 6-field Tag with 4-case compose/apply takes ~45 lines
// of repetitive field threading and forces `U64` wrapping casts on every literal.
struct Node(len: U64, sa: U64, sb: U64, sab: U64, hist: U64)
struct Tag(ta: ?U64, tb: ?U64, cab: U64, ca: U64, cb: U64, c1: U64)

const HistMonoid = Monoid<Node>(
    id: Node(0u64, 0u64, 0u64, 0u64, 0u64),
    op: (x, y) => Node(
        x.len + y.len,
        x.sa + y.sa,
        x.sb + y.sb,
        x.sab + y.sab,
        x.hist + y.hist
    )
)

// Apply order: `compose(f, g)` applies `g` first, then `f`.
// Within one tag: first accumulate `hist += cab*sab + ca*sa + cb*sb + c1*len`,
// then overwrite `a = ta` (if some) and `b = tb` (if some).
const HistAction = Action<Node, Tag>(
    id: Tag(none, none, 0u64, 0u64, 0u64, 0u64),
    compose: (f, g) => match (g.ta, g.tb):
        (none, none) => Tag(f.ta, f.tb, g.cab + f.cab, g.ca + f.ca, g.cb + f.cb, g.c1 + f.c1)
        (a1, none) => Tag(f.ta ?? a1, f.tb, g.cab, g.ca, g.cb + f.cab * a1 + f.cb, g.c1 + f.ca * a1 + f.c1)
        (none, b1) => Tag(f.ta, f.tb ?? b1, g.cab, g.ca + f.cab * b1 + f.ca, g.cb, g.c1 + f.cb * b1 + f.c1)
        (a1, b1) => Tag(f.ta ?? a1, f.tb ?? b1, g.cab, g.ca, g.cb, g.c1 + f.cab * a1 * b1 + f.ca * a1 + f.cb * b1 + f.c1),
    apply: (t, x) =>
        let nhist = x.hist + t.cab * x.sab + t.ca * x.sa + t.cb * x.sb + t.c1 * x.len
        let nsa = if let va = t.ta: va * x.len else: x.sa
        let nsb = if let vb = t.tb: vb * x.len else: x.sb
        let nsab = match (t.ta, t.tb):
            (none, none) => x.sab
            (va, none) => va * x.sb
            (none, vb) => vb * x.sa
            (va, vb) => va * vb * x.len
        Node(x.len, nsa, nsb, nsab, nhist)
)

const TICK = Tag(none, none, 1u64, 0u64, 0u64, 0u64)

fn solve() -> [Q]U64:
    // WISH A8: Offline query bucketing (`[I]List<Q>`) and answering into `[Q]U64`
    // is a recurring 6-line pattern; a stdlib `bucket_by` helper would eliminate it.
    var by_r: [I]List<Q> = [I] _ => []
    for qi in Q:
        by_r[qs[qi].r].push(qi)

    var st = LazySegTree<I, Node, Tag>.from(
        HistMonoid,
        HistAction,
        [I] _ => Node(1u64, 0u64, 0u64, 0u64, 0u64)
    )
    var st_a: List<I> = []
    var st_b: List<I> = []
    var ans: [Q]U64 = [Q] _ => 0u64

    for r in I:
        while not st_a.is_empty and a[st_a.last!] < a[r]:
            st_a.pop()
        let la = if st_a.is_empty: I.first else: (st_a.last! + 1)!
        st_a.push(r)
        st.apply(la..=r, Tag(a[r], none, 0u64, 0u64, 0u64, 0u64))

        while not st_b.is_empty and b[st_b.last!] < b[r]:
            st_b.pop()
        let lb = if st_b.is_empty: I.first else: (st_b.last! + 1)!
        st_b.push(r)
        st.apply(lb..=r, Tag(none, b[r], 0u64, 0u64, 0u64, 0u64))

        st.apply(I.first..=r, TICK)

        for qi in by_r[r]:
            ans[qi] = st.fold(qs[qi].l..=r).hist
    ans

@brute(solve)
fn solve_brute() -> [Q]U64
    where n <= 80 and q <= 80:
    var ans: [Q]U64 = [Q] _ => 0u64
    for qi in Q:
        var tot = 0u64
        for p in qs[qi].l..=qs[qi].r:
            var (ma, mb) = (0u64, 0u64)
            for k in p..=qs[qi].r:
                ma max= a[k]
                mb max= b[k]
                tot += ma * mb
        ans[qi] = tot
    ans

let res = solve()
for qi in Q: print(res[qi])

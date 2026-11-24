// Source: Codeforces 2255E2 (What Will Remain at the End? (Hard Version)),
//   https://codeforces.com/contest/2255/problem/E2
// limits: 3000 ms, 256 MB
// Problem: n, q <= 5e5 and an array a, |a_i| <= 1e9. Operations, each making a new version:
//   1 l r x (assign x, |x| <= 1e9), 2 l r (negate), 3 l r (a = max(a, 0)), 4 p (print the max
//   non-empty subarray sum of position p's values over all earlier versions). Positions are
//   XOR-encoded with the last answer mod 2^64. Multitest.
// Solution: each version appends every position's value to its history, kept as a MaxSub
//   summary. Negate and clamp see only the sign and scale with |value|, so a range tag maps
//   each sign to a new sign plus a history per unit of |value|; after an assignment the value
//   and the added history are constants. Tags compose in O(1): a SegTree with range apply and
//   point get, plus a `record` tag on the whole array after each operation, O((n + q) log n).
use std.algebra.{MaxSub}

type Sign = -1..=1

fn sign(v: Int) -> Sign = if v < 0: -1 elif v == 0: 0 else: 1

type Cell = (val: Int, hist: MaxSub)
type Path = (to: Sign, seen: MaxSub)
type Tag = (neg: Path, zero: Path, pos: Path, set: ?Cell)

fn path(f: Tag, s: Sign) -> Path = if s < 0: f.neg elif s == 0: f.zero else: f.pos

fn tag(to: Sign -> Sign, seen: Sign -> MaxSub = _ => MaxSub.zero, set: ?Cell = None) -> Tag =
    ((to(-1), seen(-1)), (to(0), seen(0)), (to(1), seen(1)), set)

fn run(f: Tag, c: Cell) -> Cell:
    let p = path(f, sign(c.val))
    let hist = c.hist + p.seen.scale(abs(c.val))
    if let s = f.set: (s.val, hist + s.hist) else: (abs(c.val) * p.to, hist)

fn then(g: Tag, f: Tag) -> Tag:
    if let s = f.set: return (f.neg, f.zero, f.pos, run(g, s))
    fn via(p: Path) -> Path = (path(g, p.to).to, p.seen + path(g, p.to).seen)
    (via(f.neg), via(f.zero), via(f.pos), g.set)

let identity = tag(s => s)
let record = tag(s => s, s => MaxSub.of(s))
let negate = tag(s => -s)
let clamp = tag(s => max(s, 0))
fn assign(x: Int) -> Tag = tag(s => s, set: (x, MaxSub.zero))

@!multitest
let (n, q): (1..=500_000, 1..=500_000) = read()
domain I = 1..=n
let a: [I]-1e9..=1e9 = read()
let unfolded = Monoid<Cell>((0, MaxSub.zero), (x, _) => x)
var st = SegTree<I, Cell, Tag>(
    a: [I] i => (a[i], MaxSub.of(a[i])),
    act: Action<Cell,
    Tag>(
        id: identity,
        compose: (g, f) => then(g, f),
        apply: (f, c, _) => run(f, c),
        over: unfolded,
    ),
)
var last: U64 = 0
fn at(y: U64) -> I = I.nth(Int((y ^ last) % U64(n)))
repeat q:
    match read<1..=4>():
        4 =>
            let ans = st.get(at(read())).hist.best
            print(ans)
            last = U64(ans)
        kind =>
            let (l, r) = (at(read()), at(read()))
            let f = match kind:
                1 => assign(read())
                2 => negate
                _ => clamp
            st.apply(min(l, r)..=max(l, r), f)
    st.apply(.., record)

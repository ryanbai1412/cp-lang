// Every version appends each position's value to that position's history, so a range tag
// is a function on (value, history summary). Negate and clamp only look at the sign, and
// scale with |value|: before the first assignment a tag maps each sign to a new sign and
// a history per unit of |value|; after it, the value is a constant and the history is fixed.
// Tags compose in O(1), so one DualSegTree (range apply, point get) handles every query.
use std.algebra.MaxSub
use std.lazy.DualSegTree

type Sign = -1..=1

fn sign(v: Int) -> Sign = if v < 0: -1 elif v == 0: 0 else: 1

type Cell = (val: Int, hist: MaxSub)
type Path = (to: Sign, seen: MaxSub)
type Tag = (neg: Path, zero: Path, pos: Path, set: ?Cell)

fn path(f: Tag, s: Sign) -> Path = if s < 0: f.neg elif s == 0: f.zero else: f.pos

fn tag(to: Sign -> Sign, seen: Sign -> MaxSub = _ => MaxSub.empty(), set: ?Cell = None) -> Tag =
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
fn assign(x: Int) -> Tag = tag(s => s, set: (x, MaxSub.empty()))

@!multitest
let (n, q): (1..=500_000, 1..=500_000) = read()
domain I = 1..=n
let a: [I]-1e9..=1e9 = read()
var st = DualSegTree<I, Cell, Tag>(
    id: identity,
    compose: (g, f) => then(g, f),
    apply: (f, c) => run(f, c),
    a: [I] i => (a[i], MaxSub.of(a[i])),
)
var last: U64 = 0
fn at(y: U64) -> I = I(Int((y ^ last) % U64(n)) + 1)
repeat q:
    let kind: 1..=4 = read()
    match kind:
        4 =>
            let ans = st.get(at(read())).hist.best
            print(ans)
            last = U64(ans)
        _ =>
            let (l, r) = (at(read()), at(read()))
            let f = match kind:
                1 => assign(read())
                2 => negate
                _ => clamp
            st.apply(min(l, r)..=max(l, r), f)
    st.apply(.., record)

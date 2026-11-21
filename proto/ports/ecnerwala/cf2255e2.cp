// Every version appends each position's value to that position's history, so a range tag
// is a function on (value, history summary). Negate and clamp only look at the sign, and
// scale with |value|: before the first assignment a tag maps each sign to a new sign and
// a history per unit of |value|; after it, the value is a constant and the history is fixed.
type Seg = (best: Int, total: Int, pre: Int, suf: Int)

fn join(a: ?Seg, b: ?Seg) -> ?Seg:
    let x = a ?? return b
    let y = b ?? return a
    (
        max(max(x.best, y.best), x.suf + y.pre),
        x.total + y.total,
        max(x.pre, x.total + y.pre),
        max(y.suf, y.total + x.suf),
    )

fn scaled(a: ?Seg, k: 0..=1e9) -> ?Seg:
    let x = a ?? return none
    (x.best * k, x.total * k, x.pre * k, x.suf * k)

fn one(v: Int) -> ?Seg = (v, v, v, v)

type Sign = -1..=1

fn sign(v: Int) -> Sign = if v < 0: -1 elif v == 0: 0 else: 1

type Cell = (val: Int, hist: ?Seg)
type Path = (to: Sign, seen: ?Seg)
type Tag = (neg: Path, zero: Path, pos: Path, set: ?Cell)

fn path(f: Tag, s: Sign) -> Path = if s < 0: f.neg elif s == 0: f.zero else: f.pos

fn tag(to: Sign -> Sign, seen: Sign -> ?Seg = s => none, set: ?Cell = none) -> Tag =
    ((to(-1), seen(-1)), (to(0), seen(0)), (to(1), seen(1)), set)

fn run(f: Tag, c: Cell) -> Cell:
    let p = path(f, sign(c.val))
    let hist = join(c.hist, scaled(p.seen, abs(c.val)))
    if let s = f.set: (s.val, join(hist, s.hist)) else: (abs(c.val) * p.to, hist)

fn then(g: Tag, f: Tag) -> Tag:
    if let s = f.set: return (f.neg, f.zero, f.pos, run(g, s))
    fn via(p: Path) -> Path = (path(g, p.to).to, join(p.seen, path(g, p.to).seen))
    (via(f.neg), via(f.zero), via(f.pos), g.set)

let identity = tag(s => s)
let record = tag(s => s, s => one(s))
let negate = tag(s => -s)
let clamp = tag(s => max(s, 0))
fn assign(x: Int) -> Tag = tag(s => s, set: (x, none))

@!multitest
let (n, q): (1..=500_000, 1..=500_000) = read()
domain I = 1..=n
let a: [I]-1e9..=1e9 = read()
let blank: Cell = (0, none)
let cells = Action(
    id: identity,
    compose: (g, f) => then(g, f),
    apply: (f, c, span) => run(f, c),
    over: Monoid(id: blank, op: (c, d) => c),
)
var st = LazySegTree<I, Cell>(cells, [I] i => (a[i], one(a[i])))
var last: U64 = 0
fn at(y: U64) -> I = I(Int((y ^ last) % U64(n)) + 1)
repeat q:
    let kind: 1..=4 = read()
    if kind == 4:
        let p = at(read())
        let ans = st.get(p).hist!.best
        print(ans)
        last = U64(ans)
    else:
        let (l, r) = (at(read()), at(read()))
        let f = match kind:
            1 => assign(read())
            2 => negate
            _ => clamp
        st.apply(min(l, r)..=max(l, r), f)
    st.apply(.., record)

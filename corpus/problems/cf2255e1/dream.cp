// Every version appends each position's value to that position's history; a query
// is the best subarray of one history. A range tag is a function on (value,
// history). Negate and clamp only see the sign and scale with |value|, so before an
// assignment a tag is, per sign, a new sign plus the history it records per unit of
// |value|; after one, the value is constant and the recorded history fixed. Tags
// compose in O(1): one dual segment tree (range apply, point get) does everything.
use std.algebra.MaxSub

enum Sign: neg = -1, zero = 0, pos = 1

fn sign(v: Int) -> Sign =
    if v < 0: Sign.neg elif v > 0: Sign.pos else: Sign.zero

struct Cell(val: Int, hist: MaxSub)       // MaxSub: stdlib Kadane monoid, scales by k >= 0
struct Path(to: Sign, seen: MaxSub)       // `seen` is per unit of |val|
struct Tag(path: [Sign]Path, set: ?Cell)  // set: value assigned, history since

fn run(f: Tag, c: Cell) -> Cell:
    let p = f.path[sign(c.val)]
    let hist = c.hist + p.seen * abs(c.val)
    if let s = f.set: Cell(s.val, hist + s.hist) else: Cell(abs(c.val) * Int(p.to), hist)

fn then(g: Tag, f: Tag) -> Tag:  // f first, then g
    if let s = f.set: return Tag(f.path, run(g, s))
    fn compose_sign(x: Sign) -> Path:
        let a = f.path[x]
        let b = g.path[a.to]
        Path(b.to, a.seen + b.seen)
    Tag([Sign] x => compose_sign(x), g.set)

fn tag(to: Sign -> Sign = x => x, seen: Sign -> MaxSub = _ => MaxSub.zero, set: ?Cell = None) -> Tag =
    Tag([Sign] x => Path(to(x), seen(x)), set)

let record = tag(seen: x => MaxSub.of(Int(x)))
let negate = tag(to: x => Sign(-Int(x)))
let clamp = tag(to: x => max(x, Sign.zero))
fn assign(v: Int) -> Tag = tag(set: Cell(v, MaxSub.zero))

@!multitest
let (n, q): (1..=5e5, 1..=5e5) = read()
domain I = 1..=n
let a: [I]-1e9..=1e9 = read()
let unfolded = Monoid<Cell>(Cell(0, MaxSub.zero), (x, _) => x)
var st = SegTree<I, Cell, Tag>(
    a: [I] i => Cell(a[i], MaxSub.of(a[i])),
    act: Action<Cell, Tag>(id: tag(), compose: (g, f) => then(g, f), apply: (f, c, _) => run(f, c), over: unfolded),
)
var last: U64 = 0
fn at(y: U64) -> I = I.nth(Int((y ^ last) % U64(n)))
repeat q:
    match read<1..=4>():
        4 =>
            let ans = st[at(read())].hist.best
            print(ans)
            last = U64(ans)
        kind =>
            let (l, r) = minmax(at(read()), at(read()))
            let f = match kind:
                1 => assign(read())
                2 => negate
                _ => clamp
            st.apply(l..=r, f)
    st.apply(.., record)

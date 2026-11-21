use std.seg.{DualSegTree, MaxSub}

// Every version appends each position's value to that position's history, so a range tag
// is a function on (value, history summary). Negate and clamp only look at the sign, and
// scale with |value|: before the first assignment a tag maps each sign to a new sign and
// a history per unit of |value|; after it, the value is a constant and the history is fixed.
type Seg = MaxSub<Int>              // best/total/pre/suf, `+` joins, `* k` scales, `Seg.empty`
enum Sign: Neg, Zero, Pos
type Cell = (val: Int, hist: Seg)
type Path = (to: Sign, seen: Seg)
type Tag = (path: [Sign]Path, set: ?Cell)

fn run(f: Tag, c: Cell) -> Cell:
    let p = f.path[Sign.of(c.val)]
    let hist = c.hist + p.seen * abs(c.val)
    if f.set is s: (s.val, hist + s.hist) else: (abs(c.val) * p.to, hist)

fn then(g: Tag, f: Tag) -> Tag:
    if f.set is s: return (f.path, run(g, s))
    ([Sign] s => (g.path[f.path[s].to].to, f.path[s].seen + g.path[f.path[s].to].seen), g.set)

fn tag(to: Sign -> Sign, seen: Sign -> Seg = _ => Seg.empty, set: ?Cell = none) -> Tag =
    ([Sign] s => (to(s), seen(s)), set)

let identity = tag(s => s)
let record = tag(s => s, s => Seg.one(s))
let negate = tag(s => -s)
let clamp = tag(s => max(s, Zero))
fn assign(x: Int) -> Tag = tag(s => s, set: (x, Seg.empty))

@!multitest
let (n, q): (1..=500_000, 1..=500_000) = read()
domain I = 1..=n
let a: [I]-1e9..=1e9 = read()
var st = DualSegTree<I, Cell>(Action(id: identity, compose: then, apply: run), [I] i => (a[i], Seg.one(a[i])))
var last: U64 = 0
fn at(y: U64) -> I = I(Int((y ^ last) % n) + 1)
repeat q:
    match read(1..=4):
        4 =>
            let ans = st[at(read())].hist.best
            print(ans)
            last = ans
        kind =>
            let (l, r) = (at(read()), at(read()))
            st.apply(min(l, r)..=max(l, r), match kind: 1 => assign(read()), 2 => negate, _ => clamp)
    st.apply(.., record)

// BTOI '06 P5 (RLE Compression): re-encode a run-length stream with the
// fewest codes by choosing when to switch the escape symbol.
// cost[e] = shortest encoding of the remaining runs when the escape is e.
// The table needs "add to all", "assign to all", "set one" and "argmin", which
// is one LazySegTree with an assign-then-add tag.
// original: corpus/submissions/4156310.cpp
(n, m): (2..=100_000, 1..=2_000_000) = read()
domain Sym = 0..n
let code: [0..m]Sym = read()

// decode into maximal runs (symbol, length)
var runs: List<(sym: Sym, len: Int)> = []
fn add_run(s: Sym, k: Int):
    if let last = runs.last and last.sym == s: runs[runs.len - 1].len += k
    else: runs.push((s, k))
var esc = Sym(0)
var at = 0
while at < m:
    let c = code[at]
    if c != esc:
        add_run(c, 1)
        at += 1
    else:
        let b = code[at + 1]
        let k = code[at + 2].idx
        if b == esc: add_run(esc, k + 1)
        elif k == 0: esc = b
        else: add_run(b, k + 3)
        at += 3

domain R = 0..runs.len
const FAR = 1e18
// tag (set, add): optionally overwrite, then add; node: (cost, symbol) minimum
type Tag = (set: ?Int, add: Int)
let assign_add = Action<(Int, Sym), Tag>(
    id: (none, 0),
    compose: (old, new) => if new.set != none: new else: (old.set, old.add + new.add),
    apply: (v, t) => ((t.set ?? v.0) + t.add, v.1))
var cost = LazySegTree<Sym, (Int, Sym)>(assign_add, (min, (FAR, Sym.last)), s => (0, s))

// switch_to[i]: escape to switch to before run i when the escape is the
// run's own symbol; adopt[i]: switch to the run's symbol from anything else
var switch_to: [R]?Sym = none
var adopt: [R]Bool = false
for i in R.rev:
    let (s, k) = runs[i]
    var x = cost.fold(s..=s).0
    cost.apply(.., (none, 3 * (k / (n + 2)) + min(3, k % (n + 2))))
    cost.apply(s..=s, (FAR, 0))
    let (opt, opt_sym) = cost.fold(..)       // cheapest other escape
    x += 3 * ((k + n - 1) / n)
    if x > opt + 3:
        switch_to[i] = opt_sym
        x = opt + 3
    if opt > x + 3:
        adopt[i] = true
        cost.apply(.., (x + 3, 0))
    cost.apply(s..=s, (x, 0))

var out: List<Int> = []
esc = Sym(0)
for i in R:
    let s = runs[i].sym
    var k = runs[i].len
    let next = if esc == s: switch_to[i] elif adopt[i]: s else: none
    if let e = next:
        out += [esc.idx, e.idx, 0]
        esc = e
    if esc == s:
        while k > 0:
            let take = min(k, n)
            k -= take
            out += [esc.idx, esc.idx, take - 1]
    else:
        while k > 3:
            let take = min(k, n + 2)
            k -= take
            out += [esc.idx, s.idx, take - 3]
        for _ in 0..k: out.push(s.idx)
print(out.len)
print(out)

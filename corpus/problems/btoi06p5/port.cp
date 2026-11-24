// Source: BOI 2006 P5 (RLE Compression), https://dmoj.ca/problem/btoi06p5
// limits: 5000 ms, 64 MB
// Problem: alphabet 0..n-1 (n <= 1e5) and a code of m <= 2e6 symbols with escape e (initially
//   0): "e e k" is k + 1 copies of e, "e b 0" makes b the escape, "e b k" (k > 0) is k + 3
//   copies of b, any other symbol is itself. Print a shortest code for the same sequence: its
//   length, then the symbols.
// Solution: decode into maximal runs, then DP backwards: cost[e] = shortest code of the rest
//   with escape e. A run of s costs the same under every escape but s; escape s pays
//   3 ceil(k / n); then s may switch (+3) to the best other escape, or all others adopt s (+3).
//   So updates are add-all, assign-all, set-one and a global argmin: one LazySegTree with an
//   assign-then-add tag and `None` for the forbidden escape. O(m + runs log n), then a
//   forward pass replays the recorded switches to emit the code.
// original: corpus/submissions/4156310.cpp
// Bounds: n: 2..=100_000; m: 1..=2_000_000; c: 0..=n - 1; b: 0..=n - 1; k: 0..=n - 1
const SWITCH_COST = 3

let (n, m): (2..=100_000, 1..=2_000_000) = read()
domain Sym = 0..n

fn escaped_cost(k: Int) -> Int = 3 * ceil_div(k, n)

fn plain_cost(k: Int) -> Int = 3 * (k / (n + 2)) + min(3, k % (n + 2))

// Decode the input stream into maximal (symbol, length) runs.
var runs: List<(sym: Sym, len: Int)> = []
fn add_run(s: Sym, k: Int):
    if let last = runs.last and last.sym == s: runs.last!.len += k
    else: runs.push((s, k))

var esc = Sym(0)
var at = 0
while at < m:
    let c: Sym = read()
    if c != esc:
        add_run(c, 1)
        at += 1
    else:
        let (b, k): (Sym, Sym) = read()
        if b == esc: add_run(esc, k.ord + 1)
        elif k.ord == 0: esc = b
        else: add_run(b, k.ord + 3)
        at += 3

domain R = 0..runs.len
type Entry = (cost: ?Int, sym: Sym)
enum SetOp: Keep | Forbid | Set(Int)
// Tag `(set, add)`: optionally overwrite (`Forbid` / `Set`), then add `add`.
type Tag = (set: SetOp, add: Int)

fn pick_min(a: Entry, b: Entry) -> Entry:
    let ca = a.cost ?? return b
    let cb = b.cost ?? return a
    if ca <= cb: a else: b

fn compose_tag(new: Tag, old: Tag) -> Tag = match new.set:
    SetOp.Keep => (set: old.set, add: old.add + new.add)
    _ => new

fn apply_tag(t: Tag, v: Entry, _span: Int) -> Entry = match t.set:
    SetOp.Forbid => (cost: None, sym: v.sym)
    SetOp.Set(c) => (cost: Some(c + t.add), sym: v.sym)
    SetOp.Keep => if let cur = v.cost: (cost: Some(cur + t.add), sym: v.sym) else: v

let cheapest = Monoid<Entry>((cost: None, sym: Sym.last!), pick_min)
let assign_add = Action<Entry, Tag>(
    id: (SetOp.Keep, 0),
    compose: compose_tag,
    apply: apply_tag,
    over: cheapest,
)
var cost = SegTree<Sym, Entry, Tag>(
    cheapest,
    a: [Sym] s => (cost: Some(0), sym: s),
    act: assign_add,
)

// switch_to[i]: escape to switch to before run i when esc == runs[i].sym;
// adopt[i]: switch to runs[i].sym from any other escape before run i.
var switch_to: [R]?Sym = None
var adopt: [R]Bool = false
for i in R.rev:
    let (s, k) = runs[i.ord]
    var x = cost.fold(s..=s).cost!
    cost.apply(.., (SetOp.Keep, plain_cost(k)))
    cost.apply(s..=s, (SetOp.Forbid, 0))
    let best = cost.fold(..)
    let opt = best.cost!
    x += escaped_cost(k)
    if x > opt + SWITCH_COST:
        switch_to[i] = Some(best.sym)
        x = opt + SWITCH_COST
    if opt > x + SWITCH_COST:
        adopt[i] = true
        cost.apply(.., (SetOp.Set(x + SWITCH_COST), 0))
    cost.apply(s..=s, (SetOp.Set(x), 0))

var out: List<Int> = []
esc = Sym(0)
for i in R:
    let s = runs[i.ord].sym
    var k = runs[i.ord].len
    let next = if esc == s: switch_to[i] elif adopt[i]: Some(s) else: None
    if let e = next:
        out += [esc.ord, e.ord, 0]
        esc = e
    if esc == s:
        while k > 0:
            let take = min(k, n)
            k -= take
            out += [esc.ord, esc.ord, take - 1]
    else:
        while k > 3:
            let take = min(k, n + 2)
            k -= take
            out += [esc.ord, s.ord, take - 3]
        for _ in 0..k: out.push(s.ord)
print(out.len)
print(out)

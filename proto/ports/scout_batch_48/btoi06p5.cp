// BTOI '06 P5 (RLE Compression): re-encode a run-length stream of symbols in
// `0..n` with the fewest codes by choosing when to switch the escape symbol.
//
// Encoding cost of a maximal run of `k` copies of symbol `s`:
// - If `esc == s` (`escaped_cost(k)`): every copy must be escaped via triples
//   `(esc, esc, take - 1)` with `1 <= take <= n` (`take - 1 in 0..n`), taking
//   `ceil(k / n)` triples = `3 * ((k + n - 1) / n)` codes.
// - If `esc != s` (`plain_cost(k)`): triples `(esc, s, take - 3)` encode
//   `4 <= take <= n + 2` copies (`take - 3 in 1..n`, since `0` switches escape)
//   in 3 codes; any remainder `r = k % (n + 2)` of `1..=3` is written literally
//   (`r` codes) and `r >= 4` takes 3 codes, so `3 * (k / (n + 2)) + min(3, r)`.
// - Switching escape from `e1` to `e2` emits `(e1, e2, 0)` (`SWITCH_COST = 3`).
//
// Backward DP: `cost[e]` is the shortest encoding of the remaining runs when
// the escape before run `i` is `e`. Each run needs "add to all", "assign to
// all", "set one", and "global argmin", maintained in one `LazySegTree` with
// an assign-then-add action and `cost: none` for forbidden entries.
// original: corpus/submissions/4156310.cpp

const SWITCH_COST = 3

let (n, m): (2..=100_000, 1..=2_000_000) = read()
domain Sym = 0..n
let code: [0..m]Sym = read()

fn escaped_cost(k: Int) -> Int = 3 * ((k + n - 1) / n)

fn plain_cost(k: Int) -> Int = 3 * (k / (n + 2)) + min(3, k % (n + 2))

// Decode the input stream into maximal (symbol, length) runs.
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
        let k = code[at + 2].ord
        if b == esc: add_run(esc, k + 1)
        elif k == 0: esc = b
        else: add_run(b, k + 3)
        at += 3

domain R = 0..runs.len
let run = [R] i => runs[i.ord]
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
    SetOp.Forbid => (cost: none, sym: v.sym)
    SetOp.Set(c) => (cost: some(c + t.add), sym: v.sym)
    SetOp.Keep => if let cur = v.cost: (cost: some(cur + t.add), sym: v.sym) else: v

let assign_add = Action<Entry, Tag>(
    id: (SetOp.Keep, 0),
    compose: compose_tag,
    apply: apply_tag,
    over: Monoid<Entry>((none, Sym.last), pick_min),
)
var cost = LazySegTree<Sym, Entry, Tag>.new(assign_add, a: [Sym] s => (some(0), s))

// switch_to[i]: escape to switch to before run i when esc == runs[i].sym;
// adopt[i]: switch to runs[i].sym from any other escape before run i.
var switch_to: [R]?Sym = none
var adopt: [R]Bool = false
for i in R.rev:
    let (s, k) = run[i]
    var x = cost.fold(s..=s).cost!
    cost.apply(.., (SetOp.Keep, plain_cost(k)))
    cost.apply(s..=s, (SetOp.Forbid, 0))
    let best = cost.fold(..)
    let opt = best.cost!
    x += escaped_cost(k)
    if x > opt + SWITCH_COST:
        switch_to[i] = some(best.sym)
        x = opt + SWITCH_COST
    if opt > x + SWITCH_COST:
        adopt[i] = true
        cost.apply(.., (SetOp.Set(x + SWITCH_COST), 0))
    cost.apply(s..=s, (SetOp.Set(x), 0))

var out: List<Int> = []
esc = Sym(0)
for i in R:
    let s = run[i].sym
    var k = run[i].len
    let next = if esc == s: switch_to[i] elif adopt[i]: some(s) else: none
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

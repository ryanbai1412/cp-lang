# apio23p2 friction

## Adding points up to a threshold is a hand-written pointer

- **Where:** `longest` sweeps `i` by `hi_i - i` and, before each query, adds every block `j`
  whose `lo_j - j` is at most that key. The port keeps `added` and a `while` over the sorted
  `js`; the idea is "the `js` up to this key", a merge of two sorted sequences.
- **Goal it hurt:** express (one fact, an offline dominance sweep, becomes index bookkeeping).
- **Routed:** stdlib worklog (a `take_through(key)` / merge-sweep over a sorted list).

## Domain positions convert to `Int` to name prefix steps

- **Where:** a position `p: I` names the step into prefix `p + 1`, so the port writes
  `Pre(Int(p) + 1)`; the block ends are `[Int(p) for p in at[v]] + [n]`.
- **Goal it hurt:** express; a `Cut`-like successor from `I` into `0..=n` would say it directly.

## Dream

```cp
let n: 1..=5e5 = read()
domain I = 0..n
domain Val = 1..=n
let a: [I]Val = read()
let at = a.positions()

struct Walk(sum: Int, lo: Int, hi: Int)
fn step(s: Int) -> Walk = Walk(s, min(0, s), max(0, s))
fn then(x: Walk, y: Walk) -> Walk = Walk(x.sum + y.sum, min(x.lo, x.sum + y.lo), max(x.hi, x.sum + y.hi))

var walk = SegTree(Monoid(step(0), then), [I.cuts] t => step(if t == I.cuts.first!: 0 else: 1))

fn longest(spans: List<(lo: Int, hi: Int)>) -> Int:
    let (domain D, tops) = compress(spans.items.map((j, s) => s.hi + j))
    var latest = SegTree<D, ?Int>(max)
    var pending = spans.keys.sorted(by: j => spans[j].lo - j)
    max(for i in spans.keys.sorted(by: i => spans[i].hi - i):
        for j in pending.take_through(key: spans[i].hi - i):
            latest.update(tops.index_of(spans[j].hi + j), x => max(x, Some(j)))
        (latest.fold(tops.first_ge(spans[i].lo + i)?..)? - i) ?? 0
    ) ?? 0

var best = 0
for v in Val:
    for p in at[v]: walk.set(p.after, step(0))
    if at[v].len > best:
        let ends = at[v].map(p => p.before) + [I.cuts.last!]
        let spans = ends.scan_with(start: I.cuts.first!, (from, to) => walk.fold(from..=to), then)
        best max= longest(spans)
    for p in at[v]: walk.set(p.after, step(-1))
print(best)
```

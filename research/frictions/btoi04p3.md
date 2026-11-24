# btoi04p3: friction report

Port: `corpus/problems/btoi04p3/port.cp`. There is no `dream.cp`; the dream is below.

## Measurements

- Tokens: port 343, accepted C++ 1090.
- Emitted C++: 23010 bytes.
- `--max` (n = 1e6, median of 15): port 236.7 ms / 14.5 MB, accepted 352.9 ms / 21.6 MB (0.67x time).
- Output: the cost line is identical (475250031304620 on the max case); the z sequence
  differs from the accepted C++ (1049447566... against 1049448535...), and both are
  optimal: the statement accepts any optimal z and `checker.py` accepts both.

## Dream

```cp
use std.heap.{Meld, MeldHeaps}

let n: 1..=1e6 = read()
domain V = 0..n
let t: [V]0..=2e9 = read()
fn x(i: V) -> Int = t[i] - i.idx
type Key = -2_000_000_000..=1_000_000  // -x

// a run of positions fitted by one median: the smallest ceil(len / 2) values, negated
struct Block(span: V.Range, low: Meld<Key>)
fn median(b: Block) -> Int = -pool.top(b.low)!

var pool = MeldHeaps<Key>(reserve: n)
var blocks = List<Block>()
for i in V:
    var b = Block(i..=i, pool.of([-x(i)]))
    while let top = blocks.last and median(top) > median(b):
        blocks.pop()
        b = Block(top.span.first..=b.span.last, pool.meld(top.low, b.low))
        while b.low.len > (b.span.len + 1) / 2: pool.pop(mut b.low)
    blocks.push(b)

print(sum(abs(x(i) - median(b)) for b in blocks for i in b.span))
for b in blocks:
    for i in b.span: print(i.idx + median(b))
```

## Friction 1: block bookkeeping restates the span three ways

- Where: `struct Block(from: V, to: V, len: Int, low: Meld<Key>)` and
  `Block(top.from, b.to, top.len + b.len, ...)`.
- What felt wrong: `len` is `to - from + 1`, kept in sync by hand at each merge; the
  median is recomputed as `pool.top(b.low)!` in four places, negated each time.
- Goal it hurts: R4 (each fact once).
- Ideal code: `span: V.Range` with `span.len`, and one `median(b)` helper (dream).
- General fix: port edit.
- Severity: low.
- Owner: idioms.

## Friction 2: a one-element heap takes a typed temporary

- Where: `let one: List<Key> = [low]` then `pool.of(one)`.
- What felt wrong: two lines to build a heap of one value. A fresh probe shows
  `pool.of([5])` on a `MeldHeaps<Int>` checks and runs (`cpl run` prints `5`), so the
  temporary is unnecessary.
- Goal it hurts: R13.
- Ideal code: `pool.of([-x(i)])`.
- General fix: port edit.
- Severity: low.
- Owner: idioms.

## Friction 3: the merge loop `break`s instead of guarding the `while let`

- Where: `while let top = blocks.last: if ... >= ...: break`.
- What felt wrong: the stop condition is the PAV invariant and belongs in the loop head.
- Goal it hurts: R2.
- Ideal code: `while let top = blocks.last and median(top) > median(b):`.
- General fix: port edit; a probe confirms `while let top = xs.last and top < 2:` checks
  and runs.
- Severity: low.
- Owner: idioms.

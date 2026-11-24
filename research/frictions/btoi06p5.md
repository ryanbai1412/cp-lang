# btoi06p5: friction report

Port: `corpus/problems/btoi06p5/port.cp`. There is no `dream.cp`; the dream is below.

## Measurements

- Tokens: port 976, accepted C++ 1867.
- Emitted C++: 29184 bytes (30959 with the hand-written tag).
- `--max` (n = 1e5, m = 2e6, median of 15, g++ -O2): port 615.8 ms / 43.8 MB, accepted
  622.9 ms / 27.6 MB (0.99x time, 1.59x memory). The hand-written tag ran 484.9 ms on the
  same input; see friction 1.
- Output: identical to the accepted C++.

## Dream

The decode and the replay are already close to the editorial. The dream differs in the
tree: the DP's four updates (add to all, forbid one, set one, set all) are one std lazy
action instead of a hand-written tag.

```cp
type Entry = (cost: ?Int, sym: Sym)
let cheapest = Monoid<Entry>((cost: None, sym: Sym.last!), (a, b) => min(a, b, by: e => e.cost))
var cost = SegTree<Sym, Entry, AssignAdd<?Int>>(
    cheapest,
    a: [Sym] s => (cost: Some(0), sym: s),
    act: assign_add.on(e => e.cost),      // hypothetical: set-then-add over one field
)
for i in R.rev:
    let (s, k) = runs[i]
    var x = cost[s].cost! + escaped_cost(k)
    cost.apply(.., add(plain_cost(k)))
    cost.apply(s..=s, assign(None))
    let best = cost.fold(..)
    ...
```

The `...` is the port's switch/adopt logic unchanged. Unverified: `assign_add`, `.on`, and
`min(by:)` treating `None` as the largest are not in std today.

## Friction 1: a hand-written assign-then-add lazy tag (fixed)

- Was: `enum SetOp: Keep | Forbid | Set(Int)`, `type Tag`, `compose_tag`, `apply_tag`,
  `pick_min` and an `Action<Entry, Tag>(...)` literal, about 35 lines, over an
  `(cost: ?Int, sym: Sym)` entry whose `None` forbade the current escape.
- Fixed: Content added `assign_add` (e9b4b0aa5), tags `(set: ?T, add: T)`. The port is now
  `SegTree(min, a: [Sym] _ => 0, act: assign_add)` over plain `Int`. The forbidden escape
  needs no `None`: `cheapest_but(s)` folds either side of `s`, and `first_at` finds the
  argmin with `right_while` only when the DP switches. Same output as the old port and
  the accepted C++.
- Cost: 615.8 ms against 484.9 ms for the hand tag (two folds per run where the tag did one
  global fold). Still 0.99x the accepted C++, so no R10 failure.
- Owner: none open.

## Friction 2: 1.76x memory

- Where: `runs: List<(sym: Sym, len: Int)>`, `switch_to: [R]?Sym`, `adopt: [R]Bool`,
  `out: List<Int>`.
- What felt wrong: the accepted C++ stores runs as two `int` arrays and writes the
  answer directly. The port keeps up to 2e6 runs with an `Int` length, an `?Sym` per run
  and the whole output list before printing; at m = 2e6 that is the 16 MB difference.
  Time is fine (0.99x), and 43.8 MB is within the 64 MB limit.
- Goal it hurts: R10 (memory).
- Ideal code: `len: 1..=2e6` in the run record (it is at most m), so the record narrows.
- General fix: port edit. Bounded record fields already narrow (probe in btoi08p6
  friction 1), so bounding `len` halves the run record.
- Severity: low.
- Owner: idioms.

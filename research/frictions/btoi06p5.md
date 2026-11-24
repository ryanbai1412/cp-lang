# btoi06p5: friction report

Port: `corpus/problems/btoi06p5/port.cp`. There is no `dream.cp`; the dream is below.

## Measurements

- Tokens: port 976, accepted C++ 1867.
- Emitted C++: 30959 bytes.
- `--max` (n = 1e5, m = 2e6, median of 15): port 498.5 ms / 46.0 MB, accepted 554.3 ms / 26.2 MB
  (0.90x time, 1.76x memory).
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

## Friction 1: a hand-written assign-then-add lazy tag

- Where: `enum SetOp: Keep | Forbid | Set(Int)`, `type Tag = (set: SetOp, add: Int)`,
  `compose_tag`, `apply_tag`, `pick_min` and the `Action<Entry, Tag>(...)` literal, about
  35 lines.
- What felt wrong: std.algebra has `add`, `assign` and `affine` actions
  (std/algebra.cp:766-774) but none composes "optionally overwrite, then add", which is
  the standard tag for range-assign plus range-add. The port builds it and its laws by
  hand, plus a `Forbid` arm because the entry's cost is optional.
- Goal it hurts: R3, "coherent stdlib".
- Ideal code: `act: assign_add` from std (dream).
- General fix: add an `AssignAddAction` to std/algebra.cp with its composition laws
  tested like the existing three (std/lazy.cp:151), over `?T` values so `assign(None)`
  is the forbid.
- Severity: medium (range assign + add is a common lazy segment tree).
- Owner: Content (std/algebra.cp).

## Friction 2: 1.76x memory

- Where: `runs: List<(sym: Sym, len: Int)>`, `switch_to: [R]?Sym`, `adopt: [R]Bool`,
  `out: List<Int>`.
- What felt wrong: the accepted C++ stores runs as two `int` arrays and writes the
  answer directly. The port keeps up to 2e6 runs with an `Int` length, an `?Sym` per run
  and the whole output list before printing; at m = 2e6 that is the 20 MB difference.
  Time is fine (0.90x), and 46 MB is within the 64 MB limit.
- Goal it hurts: R10 (memory).
- Ideal code: `len: 1..=2e6` in the run record (it is at most m), so the record narrows.
- General fix: port edit. Bounded record fields already narrow (probe in btoi08p6
  friction 1), so bounding `len` halves the run record.
- Severity: low.
- Owner: idioms.

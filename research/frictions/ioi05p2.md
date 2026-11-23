# ioi05p2 — Mountain

`spec.cp` read the command stream as a tagged enum, cut the rails into pieces with
`Rail.pieces(cuts:)` and kept (rise, peak, len) in a lazy `SegTree`. It failed `cpl check` on
two library gaps and, once those were worked around, gave wrong answers from a flipped
`compose`.

## `pieces` on a domain range rejects cuts
- **Where:** the dream line `Rail.pieces(cuts: [a.l, b.r, ...])`; today's spec.cp:
  ```cp
  let (domain P, piece) = (0..Rail.count).pieces(
      cuts: [Int(x) for c in cmds if let Cmd.Set(a, b, _) = c for x in [a.l, b.r]],
  )
  ```
- **What felt wrong:** `domain V = 0..10; V.pieces(cuts: [V(3).l])` is E0302 "expected `V`,
  found `V.cuts`" inside `std/seq.cp`, and `[V(3)]` is E0302 "can't compare a `V.cuts` with a
  `V`". So the pieces are over the rails' cut ordinals, as plain `Int`s, and `piece[p]` is a
  `Range<Int>`, not a `Range<Rail>`.
- **Goal it hurts:** express, hard-to-get-wrong (decision 162 promised no raw ints here).
- **Ideal code:** `Rail.pieces(cuts: ...)` with `piece[p]: Range<Rail>`, `piece.covering(a..=b)`.
- **General fix:** `pieces` takes `S: Seq` of `D.cuts` on a domain range (spec R04.2 pieces).
  Already routed: stdlib worklog, `covering` entry follow-up.
- **Severity:** blocker for the dream form; medium with the workaround.

## `flat_map` loses the element type of the closure's list
- **Where:** `cmds.flat_map(c => match c: Set(a, b, _) => [a.l, b.r] _ => [])`.
- **What felt wrong:** E0302 "`flat_map` does not work on the types it was given here:
  expected `Int`, found `Rail.cuts`" (`xs.flat_map(x => [V(x).l, V(x).r])` alone repeats it);
  the comprehension `[x for c in cmds if let ... for x in [a.l, b.r]]` keeps the type.
- **Goal it hurts:** express.
- **General fix:** bind `flat_map`'s yielded `V` from `U: Seq`'s element. Already routed:
  compiler worklog, "Request from stdlib" `flat_map` entry.
- **Severity:** low (the comprehension is as clear).

## A flipped `compose` type-checks and answers wrong
- **Where:** `compose: (old, new) => new ?? old` (spec.cp had the names in the wrong order;
  `compose(new, old)` is the contract).
- **What felt wrong:** the tree kept the oldest slope; the 8 old tests still passed, and 30 of 60
  generated inputs disagreed with the AC submission (one trapped on `/ 0`).
- **Goal it hurts:** hard-to-get-wrong.
- **General fix:** stress checks `laws_hold` on actions handed to `SegTree(act:)`; routed to
  tools (worklog 2026-11-23, with a 4-leaf repro).
- **Severity:** high.

## Tests 01-08 break the statement
The factory tests set slopes like `I 1 2 -8`, so elevations go negative, outside the statement's
"[0, 1e9] at any moment". They are kept; `gen.py` and tests 09-17 stay inside it (09 is the
statement sample).

## Measurements
20000-line generated input (n = 1e9): AC C++ 0.058 s, `cpl run` 1.17 s, same output.

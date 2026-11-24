# cco23p5

Score: 2/5
Dream: 3/5

Cold review against the friction's `## Dream`. The three K cases are separate and short (lines 22-32, 166-169), and the K = 2 plans are written once as lists of `Part`s that both the value (`fill`, lines 150-154) and the replay (lines 156-163) read, which removes the old port's duplicated shapes. But `top3` hand-rolls a selection std already has, unreported, which is a 2.

## Checklist failures

- R3: `top3` (lines 78-89) is an insertion sort into a 3-slot list; `kids[u].max_k(3, by: v => extra(v, done, back))` exists in `std/seq.cp` (ties to the earlier index, as the comment on line 77 wants; unordered, so sort the three). Not in the friction.
- R3, minor: `pick2`/`pick3` (lines 58-75) are argmax loops over a `found: ?(...)` with `found.map(f => gain > f.gain) ?? true`; a comprehension over the triples with `.max(by: .gain)` says the same.
- R4: the root is written twice, `root: City.first!` (line 17) and `let root = City.first!` (line 165).
- R10: 0.48 s against 0.28 s (1.7x) at `--max`; reported with a general fix (inline small lists).

## Owed rewrites

- `top3` as `max_k(3, by: ...)` (the plans only need the best first at lines 127, 132, 137: take `.max(by:)` there, or sort the three).
- `pick2`/`pick3` as `[... for x in a.top for y in b.top if x != y].max(by: .gain)`.
- Bind `let root = City.first!` before `t` and use it at line 17.

## Language gaps

- Small fixed-capacity lists for plan building (the 1.7x): in the friction.
- `cpl check` took 33 s on a closure shape in the recursive group: in the friction, reported to Core.

## Dream notes

- The dream is one memo signature (`best` returning gain and parts) and an ellipsis, judged as the port with that change. The composite keeps the hand-rolled `top3`, which is the C++ construction where std names the operation (R2), so 3; the dream should spell the top-3 with `max_k` and the plan shapes as the header's table.

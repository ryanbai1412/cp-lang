# ioi20p6

Score: 4/5
Dream: 4/5

Cold review: reads like the editorial. `Station`, `Link` and `Label` are distinct domains, the tree is std `Rooted`, the labelling is the inverse of an order table, and `next_hop` is two `find`s with the parent as the `??` fallback (lines 40-48).

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- None.

## Language gaps

- Both grader functions run in one program, and the entry/exit numbering is a hand `walk` pushing into `order` (lines 30-35): both in research/frictions/ioi20p6.md.

## Dream notes

- `near.take(near.len - (s != Label.first!))` subtracts a `Bool`; the port's `let kids = if s == Label.first!: near.len else: near.len - 1` says it.

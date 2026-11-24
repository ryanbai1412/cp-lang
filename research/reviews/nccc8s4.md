# nccc8s4

Score: 4/5
Dream: 4/5

Cold review: the label graph with an edge per pair of an atom's nonzero sides, then std `find_cycle` (lines 18-25), the header's reduction.

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- `print(if find_cycle(next) == None: 0 else: 1)` (line 25) prints a `Bool` as a digit; `Int(find_cycle(next) != None)` as in the dream says it in one value.

## Language gaps

- Negating a symmetric domain value through `Int`, and a `Graph` built straight from a dense relation: both in research/frictions/nccc8s4.md.

## Dream notes

- `joins(a, x, y)` is used but not defined; the dream should give it (two distinct nonzero sides with `x` and `-y`).

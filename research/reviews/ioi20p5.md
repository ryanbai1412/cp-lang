# ioi20p5

Score: 3/5
Dream: 3/5

Cold review: a faithful port of the two-phase strategy (brute-forced decision trees for the first 199, then interleaving with the larger pool). `Slot` and `Tree` name the pieces, but the searches are flag loops and the state is global.

## Checklist failures

- R3: `decision_tree` (lines 53-67) searches permutations with nested `loop`/`break` and an `if r not in leaves: break` flag test; a `find` over the permutations of the arrangement is the search the header describes.
- R7: `classify` advances the global `next` (line 89) and fills the global `trees` cache as a side effect of a call that reads as a query.
- R6: `side` (line 108) and `known[if res % 2 == 1: 1 - side else: side]` (line 118) carry the species as `Int`; reported as the `0..=1` species friction.

## Owed rewrites

- Lines 111-114 push the pool and the fresh mushrooms in turn by index; `known[side].take(take).zip(fresh)` flattened says the interleave once.

## Language gaps

- Interactor entry, species as an enum doubling as a bit, a memo keyed by shape, and `adjacent().count` costing 2x the index form: all in research/frictions/ioi20p5.md.

## Dream notes

- The dream is a sketch: `classify(shape(known))`, `interleave(batch)` and `counted` are left undefined, so the decision-tree search, the core of the solution, is not shown. It should name the search (`arrangements.find(separates)`) and the interleave count.

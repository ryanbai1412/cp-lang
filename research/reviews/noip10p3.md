# noip10p3

Score: 2/5
Dream: 3/5

Cold review: the angriest-first greedy is the header's, but the "must be apart" constraint is the C++ foe-pointer trick beside a `Dsu`, and std already has `ParityDsu` (`std/dsu.cp`), which noip08p4 uses.

## Checklist failures

- R3: `foe: [V]?V` with the mirrored `if let f = foe[a]: together.unite(f, b) else: foe[a] = b` (lines 17-23) hand-roll a parity DSU.

## Owed rewrites

- `var side = ParityDsu<V>()`; `side.unite(a, b)`; print the anger at the first feud that leaves it inconsistent.

## Language gaps

- None. The friction's "a parity DSU in std" is corrected in research/frictions/noip10p3.md: `ParityDsu` exists.

## Dream notes

- `if not side.apart(a, b)` both adds the constraint and tests it inside a condition, a hidden side effect; the dream should `unite` and then test `consistent`.

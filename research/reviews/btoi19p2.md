# btoi19p2 review

Score: 4/5
Dream: 3/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. Each row's possible
cells are a `Bitset<Col>`. A signal ORs in the row above or below or the row shifted a column
(all four for `?`), masks with the water, and `fold_from` runs the signals. That is the
editorial; both remaining gaps are in the friction.

## Checklist failures
- idiomatic-code/R6: signals are `Char`s tested with `heard(sig, 'N')` (lines 18, 22); in the
  friction. The enum spelling now has a run there (E0107).
- idiomatic-code/R2: the four direction cases are written per row (lines 28-31) where a grid
  bitset would shift by a step; in the friction.

## Owed rewrites
None today.

## Language gaps
- Enum variants with an input spelling (`N = 'N'`): in the friction, run added.
- A `Bitset<(Row, Col)>` with grid-step shifts: in the friction.

## Dream notes
The dream names the right objects (a grid set, a fold of shifts masked by the sea), but
`s.moves` is never defined, so the signal-to-directions map, the problem's one rule, is
missing (caps at 3). Define `moves` on `Signal`.

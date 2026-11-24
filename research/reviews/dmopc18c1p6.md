# dmopc18c1p6

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

A Knuth-optimised interval DP per column, from the last column back: `fewest[a, b]` is the fewest flips to split rows a..b, with `turn` holding the optimal split to bound the next span.

## Checklist failures

- `ones_before: [Col, Row.cuts]Int` copies every column's `Prefix` into a flat table only for speed (about 30% faster, per the friction report).
- `flips(c, a, s, b)` mixes `Int(b) - Int(s)` casts of cuts with prefix differences in one expression.
- `Row.cuts.at(Int(a) + span)` steps a cut by converting to `Int` and back.
- `var turn: [Row.cuts, Row.cuts]Row.cuts = Row.cuts.first!` fills with a placeholder every live cell overwrites.

## Gaps that need the language

- A cut can't step by k, and a table of `Prefix` is slower than a flat table: research/frictions/dmopc18c1p6.md.

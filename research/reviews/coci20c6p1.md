# coci20c6p1

Score: 4/5
Dream: 4/5

Cold review against the friction's `## Dream`. The port is the header's rule: copy the picture and, for each `#`, mark right, down and down-right, bounds checked by `Col.at`/`Row.at` (lines 15-22).

## Checklist failures

- None beyond the reported nested `if let`s (lines 19-22).

## Owed rewrites

- None.

## Language gaps

- Optional grid neighbours (`(r, c).up?.left`): in the friction.

## Dream notes

- The dream turns the scatter into a gather: a cell is `#` if it or its up, left or up-left neighbour was. One comprehension over `[Row, Col]` with no `var`; reads like the editorial.

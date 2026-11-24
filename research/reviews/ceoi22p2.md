# ceoi22p2

Score: 3/5
Dream: 4/5

Cold review against the friction's `## Dream`. The `Need(smaller, larger)` recurrence (lines 11-12, 21-22) and the closing count (line 24) are the header's; the expression is parsed by hand with two stacks, telling `min` from `max` by their middle letter (lines 13-22), which the friction reports as a missing std parser. A hand-rolled standard algorithm for a reported gap is a 3.

## Checklist failures

- R3: the two-stack parse (lines 13-22); reported.

## Owed rewrites


## Language gaps

- A small-grammar expression parser (`parse(s, grammar: ...)`): in the friction.

## Dream notes

- Judged as the port with a parsed `Expr` and `need` as a `match` over it; with the recurrence from the header that reads like the editorial.

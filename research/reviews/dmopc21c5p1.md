# dmopc21c5p1

Score: 4/5
Dream: 4/5 (the friction's `## Dream`)

Scored cold against the checklist and the port.

A construction: the odd numbers except 5, then 5 and 4, then the even numbers except 4, so each adjacent sum is composite; n = 1 and n <= 4 are handled first.

## Checklist failures

- `[5, 4]` is the construction's hinge but nothing says why those two; the editorial's reason (5 + 4 = 9) is not in the code.
- `var order: List<1..=1e6>` is built by three `extend`s; one concatenation of three comprehensions would make it a `let`.

# noip99p2

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

## Checklist failures

- `[if c.is_digit: c - '0' else: c - 'A' + 10 for c in number]` parses base-N digits by hand.
- `mirror_sum` keeps digits as a `List<Int>` with a running `carry`, a hand-written bignum add in base `base`.
- `steps() -> ?0..=30` with `return k` inside a `for` and `None` after it is a find over `0..=30` written as a loop.

## Gaps that need the language

- Base-N digits parsed by hand: research/frictions/noip99p2.md.

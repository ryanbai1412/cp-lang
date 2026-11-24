# dmopc21c5p3

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

A prime-subtraction game table `win` over 1..=MAX_N, then per test the identity or the identity with a short reversed tail.

## Checklist failures

- `n == 11 or n == 36 or n == 311` are three magic exceptions with no comment on where they come from.
- `win` is filled by a nested loop with `if p > i: break` and a second `break`; `win[i] = any(not win[i - p] for p in sieve.primes.take_while(p => p <= i))` says the rule.
- `(1..=n - tail.len).collect() + tail` builds the answer by list arithmetic.

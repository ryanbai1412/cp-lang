# ccc19s2

Score: 2/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header states the sieve bound and the smallest-a choice.
- R2: sieve, then the first prime a with 2n - a prime: the editorial.
- R3: the port builds its own `is_prime` list from `primes_in` (L13-14) although std's `Sieve(n)` keeps both `primes` and `is_prime`, as the dream uses; the report says "No gap".
- R4: `2 * ns.max()! + 1` is written twice (L13-14).
- R10: 0.02 s vs 0.00 s, both start-up.

## Owed rewrites

- `let sieve = Sieve(2 * ns.max()!)` and query it, as the dream does; correct the report's "No gap".

## Language gaps

None.

## Dream notes

The editorial in a few lines.

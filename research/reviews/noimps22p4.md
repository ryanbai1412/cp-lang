# noimps22p4

Score: 3/5
Dream: 2/5

Cold review on main 6a989896d: port, dream and friction report, not the C++. SOS over small-prime masks per big prime and inclusion-exclusion over the query's submasks follow the header, with std `sos` and `submasks`. Masks stay raw `Int`, and the dream is an outline, so the score is provisional until the dream is rewritten.

## Checklist failures

- Lines 36-39 and 55-58 split a prime's rank into a mask bit or a big-prime row twice (idiomatic-code/R4); the typed split is reported (friction 2).
- Line 23 `let smalls = 13` is the 13th prime from the header, written as a literal rather than counted from `primes` (idiomatic-code/R4).
- Masks are `Int` with `1 << j`, `full ^ s` and `popcount(s)` (lines 34-70) rather than a `Subset` over the small primes (idiomatic-code/R5).
- Lines 45-46 build the powers of two by a push loop.

## Owed rewrites

- Rewrite the dream as code: it has placeholders (`sos([count of cards with mask m])`, `sos(...)`) and undefined `part`, `free` and `sign`.

## Language gaps

- Prime rank from a sieve: friction 1.
- Small/big split as an enum: friction 2.

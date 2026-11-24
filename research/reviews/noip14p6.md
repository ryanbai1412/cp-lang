# noip14p6

Score: 4/5
Dream: 4/5

Cold review: sieve the candidates by f mod three small primes, then check the survivors modulo 2^61 - 1 (lines 19-31), the header's argument, now that a local `Mod(p)` parses.

## Checklist failures

- R1: the header's "at most n roots" mod p (line 7) fails when p divides every coefficient, and "about n / 2^61 per x" (line 10) is not a probability for a fixed modulus; the cost and error claims need that caveat.

## Owed rewrites

- `print(roots, sep: "\n")` (as in dream.cp line 17) in place of the `for` (line 31).

## Language gaps

- None open; the local `Mod(p)` read fixed in research/frictions/noip14p6.md.

## Dream notes

- None.

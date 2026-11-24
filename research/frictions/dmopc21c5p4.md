# Friction report: dmopc21c5p4

No gap. The pair `up + down = total` is `sieve.primes.find(...)!` (Goldbach guarantees it), in place of a function returning a `(0, 0)` sentinel, and the tests are `@!multitest`. Many permutations are optimal, so `checker.py` checks prime gaps and the count of distinct differences.

## Measurements

gen.py --max (N = 1e6 - 20 plus N = 1..=20): port 0.08 s, accepted C++ 0.05 s.

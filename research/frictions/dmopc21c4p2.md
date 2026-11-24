# Friction report: dmopc21c4p2

No semantic gap. The port walks the sizes up and covers the multiples of each group minimum, so it needs no sieve or divisor lists. That is simpler than `dream.cp` and `spec.cp`, which test the divisors of each value against the kept set.

## Measurements

gen.py --max (N = 2e6, J = 5e6): port 0.13 s, accepted C++ 0.49 s.

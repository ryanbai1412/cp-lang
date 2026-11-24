# Friction report: dmopc21c5p3

No gap. The bound is one `const MAX_N`, the win table is computed once before `@!multitest`, and the identity answer is `(1..=n).collect()`.

## Measurements

gen.py --max (47 tests near n = 1e5, plus 11, 36, 311): port 0.46 s, accepted C++ 1.27 s.

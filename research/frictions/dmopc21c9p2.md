# Friction report: dmopc21c9p2

No gap. Bare `@!multitest`, the two strings read separately, `pending.first == Some(b[matched])` in place of an emptiness check plus `pending[0]`, and `yes(...)` for the answer word.

## Dream

The port as written. A deque of letters merges its last two while equal and below z (`pending.push(x + 1)`), and pops matched letters of B from the front.

## Measurements

gen.py --max (one puzzle, |A| = |B| = 5e5): port 0.00 s, accepted C++ 0.00 s.

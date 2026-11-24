# abc481g

Score: 4/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp`, without the C++. The header derives the per-gap
generating function `dv 2^(n-q) x^p (1 + x)^q` and the divide and conquer that factors out
`x^p_min (1 + x)^q_min`. The code follows it with `Poly<Z>` products and std `Comb`, and an
`@alt(program)` brute force.

## Checklist failures
- idiomatic-code/R13: `min(a, b)` and `max(a, b)` (lines 36-37) on a pair the statement orders.
  The friction reports it and says the dream's `cakes[i].a`/`.b` lines pass, but the port still
  has them.
- idiomatic-code/R2: the `cur_p`/`cur_q` counters (lines 41-51) are a running scan kept by hand
  across the event loop. Minor: the header names p and q.

## Owed rewrites
- Take the dream's `Event(cakes[i].a, End.Low)` / `Event(cakes[i].b, End.High)` lines (the
  friction measured them: same output, 0.176 s).

## Language gaps
- The order a_i < b_i as a dependent field bound: in the friction, with a run.

## Dream notes
Reads like the editorial; it keeps the hand-carried `cur_p`/`cur_q` and the
`times_x` helper built from `List.filled` zeros.

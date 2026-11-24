# agc079a

Ported from `orig.cpp` (the AtCoder editorial idea; no accepted submission in the package).
Dream: `corpus/problems/agc079a/dream.cp`.

## The dream reads wider bounds than the statement
- **Where:** `dream.cp` lines 8-9: `let n: 2..=3e5 = read()` and `let p: [0..n - 1]1..=n = read()`.
- **What felt wrong:** the statement (quoted in `gen.py` line 3) says `2 <= N <= 250000` and
  `1 <= P_v < v`. The dream allows n up to 3e5 and P up to n, so it states neither bound, and
  the port's `2..=250_000` / `1..=n - 1` are the right ones. The dream passes `verify` only
  because the generated inputs are inside both.
- **Goal it hurts:** types carry the proofs (idiomatic-code/R5); the dream is the standard ports
  are judged against.
- **Ideal code:** `let n: 2..=250_000 = read()` and `let p: [0..n - 1]1..=n - 1 = read()`. The
  bound `P_v < v` (here `p[i] <= i + 1`) is per element, which no read type states, so it
  stays in the header.
- **General fix:** edit `dream.cp` to the statement's bounds.
- **Severity:** low.
- **Owner:** idioms (this package). It is a `.cp` edit, and the `copies` slice that the
  pre-commit maps to it is red on main (routed to Perf C by Factory), so it lands with the
  abc481g port fix.

## Port and dream differ, both fine
The port streams `p` and builds `num = (n - 1)!` in the same loop. The dream reads `p` whole and
takes `comb.fact(n - 1)`. They are the same algorithm. `Comb` precomputes inverse factorials up
to its size, which this problem never uses. The port's single loop is the shorter spelling
here, and both are 4/5 in the review.

## Measurements
- Tokens: port 141, `orig.cpp` 11,459 (a template-heavy contest file).
- `--max` (one test, sum N = 2.5e5; best of 3): port 7.6 ms / 3.1 MB, `orig.cpp` 15.6 ms /
  3.4 MB. Same output.
- Emitted C++: 13,704 bytes.

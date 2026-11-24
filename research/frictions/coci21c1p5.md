# coci21c1p5 friction

The old port read `n` and the permutation as plain `Int`, kept per-level cursors in a raw
`List<Int>` indexed by `chain.len`, special-cased `h == 0` instead of asking for the chain's
top, and failed `cpl check` (E0603 on `let cand = level[h][next[h]]` then
`chain.push(cand)`). The port types the permutation as `[Pos]1..=n`, gives the levels their
own `Level` domain, and branches on `chain.last`. The problem had no generator; `gen.py`
and `checker.py` are new (the chains are not unique).

## Dream

The existing `dream.cp` is the target; the port matches it except for the two lines below.

## Building a domain-indexed table from a growing list copies it

- **Where:** the levels are found while reading (their count is the LIS length), so they
  grow in a `List<List<Pos>>` and become `[Level]List<Pos>` afterwards with
  `[Level] h => on_level[h.ord].clone()`: a second copy of every level. A move out of the
  list (`on_level.into([Level])` or a `List.take()`) would avoid it.
- **General fix:** an owned conversion from `List<T>` to `[D]T` when `D.count == len`.
- **Status:** open.

## The checked-in port failed `cpl check`

- E0603 on a plain `Pos` read out of a nested list and pushed; sent to Core with the
  repro `var xs: List<List<Int>> = [[1]]` / `let x = xs[0][0]` / `ys.push(x)`. The
  rewrite's `let at = level[h][used[h]]` then `chain.push(at)` checks on today's compiler.

Measured: `cpl stress` with `checker.py` agrees with the accepted C++ (4086660) on 30 cases
(size 10) and 60 cases (size 60); `checker.py` accepts the port's `--max` output (n = 1e6),
0.16 s against the C++'s 0.27 s.

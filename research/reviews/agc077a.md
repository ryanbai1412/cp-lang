# agc077a

Score: pending (port keeps the old shape until a reversed list view stops pulling 4.9 KB of `list_view` runtime; the rewrite grows emitted C++ 18.9 -> 23.2 KB while user code shrinks 5.1 -> 3.9 KB)
Dream: 3/5

Dream scored cold on main 94953ccd. Old 2/5, new 3/5; sent back. Viewing each block four ways through `Run`/`Block.each` (lines 35-46) reads like the editorial, and the header derives the legal side and the n / 2 bound. Owed: (1) line 73 prints `op.span.lo + 1, op.span.hi`, shifting only one end of an inclusive 0-based span; either it is off by one or the output convention needs stating. A `Pos` domain with a 1-based io base would remove the hand shift; (2) the before/after branches (lines 60-65) mirror each other; name the side (`enum Side`) and pick the list and the flipped view from it once; (3) `bs_x`/`bs_y` (lines 31-32) build the same prefix twice; one `[x, y]`-indexed prefix or a helper says it once. No friction report exists for agc077a; the emitted-size blocker in the review header needs one. Earlier notes below are superseded where they conflict.

Re-reviewed after fbceb1d1, port and dream together (the port is the dream reformatted). `Pos`, `prefix1` and one `count_b` replace the duplicated getters. But four `Seg`s are moved in lockstep through tuple assignments, positions are raw `Int` with `+ 1` into `Pos`, and the recursion pushes into global op lists. The dream is under 4, so it goes back and the port score is provisional. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Dream failures

- Retained C++ structure (dream lines 47-63): `x`, `pre`, `post`, `y` are four parallel segments; every step is a four-way tuple assignment (lines 49, 53-54, 56). The editorial object is one pair of aligned strings with a pending orientation.
- Raw positions (lines 10-15, 42, 58, 61, 65-66): `Seg` holds `Int` with a direction `d`, then `Pos(s.l + 1)` at every use; `Pos` is declared but not carried.
- Direction case split (line 42): `count_b` needs `if s.d == 1` because `Seg` is not a `Pos` range.
- Hidden side effects (lines 44-45, 58, 61): `solve_seg` appends to globals.
- Character tests (lines 34, 48, 51, 55): raw `'A'`/`'B'`; `Int(bool)` prefixes (lines 38-39) where `prefix1` over `xs[p] == 'B'` counts directly.
- Header (lines 2-7) names `drop(ma)`/`take(ma)` but not why one reversal per balanced block suffices.

## Owed rewrites

- `Seg` over `Pos` with `.rev` views; one aligned pair instead of four segments; return ops from `solve_seg`; `enum Letter`.

## Language gaps

No friction report exists. Possibly owed: a reversible `Pos` range view (`lo..=hi` with `.rev`) usable as a prefix-sum query.

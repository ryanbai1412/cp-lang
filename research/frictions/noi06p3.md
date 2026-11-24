# Friction report: noi06p3

The dream is the comb DP stated over every depth:
- `@memo(array) fn best(i: Col, h: H, s: Side, past: Bool)` with `domain H = 0..=depth.max()! + 1`;
- `past` steps to a strictly deeper (High) or shallower (Low) depth.

The port keeps the same states (`lo`/`hi` per row and height). It tries only heights within 2 of a value in rows i - 2..=i + 2, at most 25 per row, and replaces `past` with running minima over the next row's sorted candidates.

This review also fixed one owed rewrite in the port. The input used to be read row by row into `var a`/`var b` and mirrored in a second loop. It is now the dream's `read<[Col](l:, r:)>()` with `rows.map`.

## The dream's state space does not fit the bounds
- **Where:** `dream.cp` `best` over `Col x H x Side x Bool`; with n and the depths up to 1e6, that is about 4e12 states. The port's `cands_at` and the two monotone sweeps in `solve_env` (lines 31-62) are what make it O(25 n).
- **What felt wrong:** the dream states the recurrence, but the observation that only near-by heights matter has to be written as hand-built candidate lists of `Int` heights, with `q` cursors and `List<?Int>` DP rows indexed by position, not by height.
- **Goal it hurts:** express.
- **Ideal code:** `best` memoized over `(i, h)` with `h` drawn from a per-row candidate set (`@memo(map)` or a `[Col]SortedMap<Int, Int>` row), and the `past` transition written as `row.fold(after: h)` / `row.fold(before: h)` prefix minima.
- **General fix:** a std "DP row keyed by sorted candidates" with prefix/suffix min queries would remove the two pointer loops. Owner: stdlib.
- **Severity:** low (the candidate set is the algorithm; only its spelling hurts).

## Measurements

- **Tokens:** port 575, accepted C++ 854 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.63 s / 48 MB, accepted C++ 2.04 s / 632 MB, with the same output.
- **Emitted C++:** 21161 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.

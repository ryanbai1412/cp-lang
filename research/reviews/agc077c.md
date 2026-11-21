# agc077c

Score: 2/5
Dream: pending cold rescore (rewritten: proves `sum(out_deg) = n(n-1)/2 ≡ floor(n/2) (mod 2)` parity invariance + even/odd `k` cases in header, replaces list-comprehension `.len` with `comp.count(v => odd_out[v])`)

The complement-graph components are found by a hand DFS with a two-pointer skip over sorted adjacency, and the parity bookkeeping (`out_deg`, `c0`/`c1`, `mi_1`/`ma_1`) is never explained. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- No invariant header: why even `k` is always yes, what `out_deg` parity means, and why the answer is `n / 2` inside `[mi_1, ma_1]`.
- Raw index space (lines 3-7): `[0..n]` with `u0 - 1`; no vertex domain.
- Unexplained initial parity (line 4): `(n - 1 - i) % 2`, and the `u > v` flip (lines 10-12).
- Hand complement traversal (lines 26-35): `unvisited.first_ge`, a manual adjacency pointer `ai` and recursion over raw `Int`.
- Paired counters `c0`/`c1` and unexplained closing terms (lines 38-39): `c1 % 2`, `tot - (tot - c1) % 2`.

## Owed rewrites

- Header with the parity argument; a `V` domain; name the complement components; derive the per-component range.

## Language gaps

No friction report exists. Possibly missing: complement-graph components in `std.graph`; needs a fresh-compiler check before filing.

## Cold review (HEAD)

Dream 3/5 (new, d93c0286), sent back.
- Line 44 checks only `lo <= target <= hi`, but line 11 says each component reaches only counts *of the same parity as `odd`*. Either the parity of `target` is forced, and the header must say why, or the check is missing it.
- Lines 7-8 state the k-even and k-odd conditions without deriving either.
- `[v for v in comp if odd_out[v]].len` (line 39) should be `comp.count(v => odd_out[v])`, which works today.
- `complement_components` (line 37) isn't in the stdlib (E0201); the friction report covers it.

Port 2/5: it hand-rolls the complement-graph DFS with an unvisited set, a `cursor` and a merge pointer `ai` (the reported gap).

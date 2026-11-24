# yac6p6

Score: 3/5
Dream: 4/5

Cold review of `port.cp` against `dream.cp`, without the C++. The header derives the
anti-diagonal pivot (every path crosses `x + y = d` once unless the subgrid is small), the
buckets, the two walks per pivot, the split of the zeros, and the cost. The port departs from the
dream's per-cell `[KDom]?Int` tables for flat `[Row, Col, Zeros]Int` ones because the per-cell
table heap-allocates (1.16 s at max against a 1000 ms limit, in the friction), which is the
rubric's measured-too-slow 3. With L = 16 pivots it runs 0.48 s at --max against the
accepted C++'s 0.64 s (idiomatic-code/R10 met; the old 3-cell spacing ran about 1.7 s).

## Checklist failures
- idiomatic-code/R1: the header says `Q <= 1e5` (L3) while `// Bounds:` and the read say
  `q: 1..=200_000` (L14, L19): one of the two is wrong. Unreported.
- idiomatic-code/R2: the pay-or-skip step is written twice, in `walk_back` (L44-45) and
  `walk_ahead` (L61-62), through the direction helpers `onward`/`before`, where the dream has one
  `via(next, k, cost)`. Reported: forced by the flat tables.
- idiomatic-code/R4: `Int(x2) + Int(y2)` is written twice in `d` (L70; also in the dream).

## Owed rewrites
- Make L3 agree with the statement's bound on q.
- Bind the end's diagonal once before rounding it down to a multiple of `L` (port and dream).

## Language gaps
- A constant-size domain table `[Zeros]Int` lowers to a heap `std::vector`, measured 1.16 s
  (performance; in the friction).
- `g[r, c, k]` and `g[r, c][k]` on a `[R, C][Zeros]Int` each rejected, pointing at the other
  (E0305/E0503; in the friction).

## Dream notes
The dream states the pivot invariant in its header, uses `?Int` for unreachable cells and one
`via` for both directions. The repeated diagonal sum is the one fact written twice; 4/5.

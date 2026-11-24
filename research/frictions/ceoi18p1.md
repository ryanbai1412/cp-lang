# ceoi18p1: friction report

Port: `corpus/problems/ceoi18p1/port.cp`, dream: `corpus/problems/ceoi18p1/dream.cp`.

## Measurements

- Tokens: port 244, accepted C++ 475.
- Emitted C++: 15057 bytes.
- `--max` (n = m = 2000, median of 15): port 311.4 ms / 3.6 MB, accepted 290.4 ms / 4.1 MB (1.07x time).
- Output: identical to the accepted C++.

## Friction 1: deals are positional tuples ordered by a sign trick

- Where: `[(o.rate, o.cores, -o.price) for o in machines] + [(o.rate, -o.cores, o.price) for o in orders]`,
  `deals.sort()`, `for (_, cores, gain) in deals.rev`.
- What felt wrong: "fastest first, machines before orders at equal rate" is encoded as
  ascending tuple order walked in reverse, where machines win ties because their core
  count is positive. The dream says it: named fields and
  `.sorted(by: d => (desc(d.clock), d.order))`.
- Goal it hurts: R2, R4 (the tie rule lives in the sign of a field).
- Ideal code: the dream's `deals`.
- General fix: port edit.
- Severity: medium (the tie rule is the one subtle step of the problem).
- Owner: idioms.

## Friction 2: the knapsack bound and the final max are hand-rolled

- Where: `let top = 50 * n`, `var best: [0..=top]?Int`, and
  `var profit = 0; for b in best: profit max= b`.
- What felt wrong: `top` restates the cores bound times n where the dream sums the
  machines' cores (`Free = 0..=comps.sum(of: .cores)`), and the final loop folds `?Int`
  into an `Int` by hand where `best.present().max()!` says it.
- Goal it hurts: R4, R8.
- Ideal code: `domain Free = 0..=machines.sum(of: .cores)` and `print(best.present().max()!)`.
- General fix: port edit.
- Severity: low.
- Owner: idioms.

The two directions of the 0/1 update (upward for orders, downward for machines) stay
explicit in the port; the dream rebuilds the table each deal, which is simpler but
allocates per deal and is the reason the port, not the dream, is the one to measure.

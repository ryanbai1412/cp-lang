# btoi19p4 review

Score: 4/5
Dream: 3/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. The header proves
the feasibility condition (cover sum A, and sum of min(B[j], n) >= n k) and turns it into a 0/1
knapsack over exact paid hours. The port is that knapsack over a `Paid` domain plus one
`Paid.find`. Both remaining gaps are in the friction.

## Checklist failures
- idiomatic-code/R3: the knapsack is hand-written (lines 25-30); in the friction (std has no
  0/1 knapsack keeping the best value per exact weight).
- `(variety[p] ?? 0) >= n * k` (line 32) works around unordered `?Int`, as the friction says.
  It is safe here since n k >= 1. The run is now in the friction (E0308).

## Owed rewrites
None today.

## Language gaps
- Ordered optionals: in the friction, run added.
- A std 0/1 knapsack: in the friction.

## Dream notes
The two lines name the right pieces (`knapsack_max`, `Paid.find`), but they drop the
`any(a < k for a in need)` impossibility check and compare `p >= work` with `p` a `Paid` and
`work` an `Int`. It is a sketch, not the program (caps at 3).

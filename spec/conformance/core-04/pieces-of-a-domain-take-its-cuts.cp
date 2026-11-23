// expect: ok, output "3 4 3"
// rule: 04.2.28
// `D.pieces(cuts:)` on a domain takes `D.cuts`, the domain's own cut positions; inputs are cuts, pieces are ranges (decision 162).
domain Rail = 0..10
let (domain P, piece) = Rail.pieces(cuts: [Rail.cuts.nth(3), Rail.cuts.nth(7)])
print([piece[p].len for p in P])

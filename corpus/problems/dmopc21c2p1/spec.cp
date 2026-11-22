// dmopc21c2p1: pay a per unit to lower the water level to i, then b per unit for every column
// still above it. The cost is a sum of convex functions of i, so ternary search finds the best.
let (n, a, b) = read<(1..=200_000, 0..=1_000_000, 0..=1_000_000)>()
let h = read<[n]0..=1_000_000>()
fn cost(i: Int) -> Int = a * i + b * h.sum(of: x => max(x - i, 0))
print(cost(ternary_min(0..=h.max()!, cost)!))

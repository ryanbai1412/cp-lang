// DMOPC 2021 C2 P1: choose a level i ≥ 0 paying a per unit of i and b per unit each value
// still sits above i. The cost is convex in i, so ternary search it.
let (n, a, b) = read (Int, Int, Int)
let xs = read [n]Int
print(ternary_min(0..=xs.max()!, i => a * i + b * xs.sum(of: x => max(x - i, 0))).value)

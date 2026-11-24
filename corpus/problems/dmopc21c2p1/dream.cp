// DMOPC 2021 C2 P1: buy L permanent power up front for h each, then p per missing unit at each
// boss. The cost is linear plus a sum of convex hinges in L, so it is convex: ternary search it.
let n: 1..=1e6 = read()
let h: 1..=1e6 = read()
let p: 1..=1e6 = read()
let boss: [0..n]0..=1e6 = read()
fn cost(l: Int) -> Int = h * l + p * boss.sum(of: a => max(a - l, 0))
print(cost(ternary_min(0..=boss.max()!, cost)!))

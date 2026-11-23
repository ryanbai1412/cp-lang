// CF 1862D Ice Cream Balls
// in: t; n (1..1e18) | out: per test the fewest balls giving exactly n distinct two-ball cones
@!multitest
let n: 1..=1e18 = read()
let x = (1..=2e9).last_true(x => x * (x - 1) / 2 <= n)!
print(x + (n - x * (x - 1) / 2))

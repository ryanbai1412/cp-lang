// CSES 1653 Elevator Rides: fewest rides (capacity x) to carry n <= 20 people.
let (n, x): (1..=20, 1..=1e9) = read()
domain P = 0..n
let w = read<[P]1..=1e9>()
// best (rides, weight in the last ride) once the people in s are up
var dp: [P.subsets]?(Int, Int) = None
dp[Subset<P>.empty] = (1, 0)
for s in P.subsets:
    let (rides, last) = dp[s]!
    for p in P if p not in s:
        dp[s.with(p)] min= if last + w[p] <= x: (rides, last + w[p]) else: (rides + 1, w[p])
print(dp[Subset<P>.full]!.0)

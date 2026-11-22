// CSES 1653 Elevator Rides: fewest rides (capacity x) to carry n <= 20 people.
let (n, x): (1..=20, 1..=1e9) = read()
domain P = 0..n
let w: [P]1..=1e9 = read()
type Mask = Subset<P>
// best (rides, weight in the last ride) once the people in mask are up
var dp: [Mask]?(Int, Int) = none
dp[Mask.empty] = (1, 0)
for mask in Mask:
    let (rides, last) = dp[mask]!
    for p in P if p not in mask:
        dp[mask.with(p)] min= if last + w[p] <= x: (rides, last + w[p]) else: (rides + 1, w[p])
print(dp[Mask.full]!.0)

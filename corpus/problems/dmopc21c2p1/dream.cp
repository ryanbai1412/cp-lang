// dmopc21c2p1: pay a per unit to lower the water level to i, then b per unit for every column
// still above it. The cost is convex and piecewise linear in i with breaks at the heights, so
// only i = 0 and the heights need checking, each in O(log n) with prefix sums.
let (n, a, b) = read<(1..=200_000, 0..=1_000_000, 0..=1_000_000)>()
let h = read<[n]0..=1_000_000>().sorted()
let pre = h.scan(+)
fn cost(i: Int) -> Int:
    let c = h.first_gt(i) ?? n
    i * a + b * (pre[n] - pre[c] - (n - c) * i)
print(chain([0], h).map(cost).min()!)

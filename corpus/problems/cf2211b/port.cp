// Source: Codeforces 2211B (Mickey Mouse Constructive),
//   https://codeforces.com/contest/2211/problem/B
// limits: 1500 ms, 256 MB
// Problem: f(a) counts the partitions of a into subarrays that all have the same sum. Over
//   arrays of x copies of 1 and y copies of -1 (x, y <= 2e5), print the least f(a) mod
//   676767677 and an array reaching it. Multitest, sums of x and of y <= 2e5.
// Solution: the total is x - y, and a partition into d parts needs d | |x - y|; prefix sums
//   move by 1, so they pass every k(x - y)/d and each divisor gives at least one partition.
//   Putting the minority sign first sends the prefix sums away from the total and then
//   straight to it, so each divisor gives exactly one: f = divisors(|x - y|), or 1 when
//   x = y (no inner prefix sum is 0). O(sqrt(|x - y|)) plus the output.
// Bounds: x_in, y_in: 0..=200_000

const MOD = 676_767_677
type Z = Mod<MOD>

@!multitest
let (x_in, y_in): (0..=200_000, 0..=200_000) = read()
let (x, y) = (min(x_in, y_in), max(x_in, y_in))
let vx = if x_in > y_in: -1 else: 1

let cnt: Z = if x == y: 1 else: divisors(y - x).len
print(cnt)
var out: [0..x + y]Int = -vx
out[0..x] = vx
print(out)

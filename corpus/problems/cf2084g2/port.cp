// Source: Codeforces 2084G2 (Wish Upon a Satellite, Hard Version),
//   https://codeforces.com/contest/2084/problem/G2
// limits: 4000 ms, 500 MB
// Problem: f(c) is the end value of a game on c: players alternately merge two adjacent entries,
//   Turtle (first, maximising) into their min, Piggy (minimising) into their max, until one is
//   left. The beauty of a permutation sums f over all subsegments. Fill the zeros of a partial
//   permutation of n <= 5e5 to maximise the beauty; print it. Multitest, sum of n <= 5e5.
// Solution: the beauty depends only on which values sit at even positions: it is
//   (n(n+1)^2/2 + sum over k of gain(k, c)) / 2, where c counts the values <= k at even positions
//   and gain is (evens - odds among values <= k) * (evens - odds among values > k). Place values
//   1..n in order; dp(c) is concave: each step adds the concave quadratic gain(k, .), then a free
//   value takes max(dp(c), dp(c - 1)) (max-convolve with a flat 0..=1) and a fixed one shifts by
//   its parity. Slope trick (Concave, a lazy treap of slopes): O(n log n); answer at c = n / 2.
// Bounds: n: 1..=500_000
@!multitest
use std.slope.{Concave}

let n: 1..=500_000 = read()
domain Pos = 1..=n
let a: [Pos]?(1..=n) = read(none: 0)
let half = n / 2

var fixed_even: [1..=n]?Bool = None
for i in Pos:
    if let v = a[i]:
        fixed_even[v] = i.label % 2 == 0

fn gain(k: Int, c: Int) -> Int = (2 * c - k) * (2 * (half - c) - (n - k))

var dp = Concave.point(0)
dp.reserve(n)
for k in 0..n:
    dp.add_quadratic(c => gain(k, c))
    if let is_even = fixed_even[k + 1]:
        dp.max_convolve(Concave.point(Int(is_even)))
    else:
        dp.max_convolve(Concave.flat(0..=1))

print((dp.at(half) + n * (n + 1) * (n + 1) / 2) / 2)

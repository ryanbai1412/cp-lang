// Codeforces 2084 G2: Wish Upon a Satellite (Hard Version).
// dp(c) over the number of evens c is concave: each step adds the quadratic
// gain, and a free value takes max(dp(c), dp(c - 1)), a 0 slope merged in.
@!multitest
use std.slope.{Concave}

let n: 1..=500_000 = read()
domain Pos = 1..=n
let a: [Pos]?(1..=n) = read(none: 0)
let half = n / 2

var fixed_even: [1..=n]?Bool = none
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

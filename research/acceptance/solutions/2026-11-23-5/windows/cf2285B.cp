// B. Diverse String
// https://codeforces.com/contest/2285/problem/B
fn tri(x: Int) -> Int = x * (x + 1) / 2
@!multitest
let (n, k): (1..=2e5, 0..=2e5) = read()
let s: Str = read()
let ones = s.count(c => c == '1')
fn div(c: Int) -> Int = 1 + tri(c) + tri(n - c)
print(max(div(max(0, ones - k)), div(min(n, ones + k))))

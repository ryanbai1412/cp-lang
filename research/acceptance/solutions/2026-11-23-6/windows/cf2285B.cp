@!multitest
let n: 1..=2e5 = read()
let k: 0..=n = read()
let s: Str = read()
let ones = s.count(c => c == '1')
fn f(c: Int) -> Int: 1 + c * (c + 1) / 2 + (n - c) * (n - c + 1) / 2
let lo = max(0, ones - k)
let hi = min(n, ones + k)
print(max(f(lo), f(hi)))

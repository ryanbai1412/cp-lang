@!multitest
let (n, k): (1..=2e5, 0..=2e5) = read()
let s: Str = read()
let ones = s.count(c => c == '1')
fn div(c: Int) -> Int = 1 + c * (c + 1) / 2 + (n - c) * (n - c + 1) / 2
print(max(div(max(ones - k, 0)), div(min(ones + k, n))))

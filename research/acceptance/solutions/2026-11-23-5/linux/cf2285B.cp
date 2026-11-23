@!multitest
let (n, k): (1..=2e5, 0..=2e5) = read()
let s: Str = read()
let ones = s.count(c => c == '1')
let big = min(n, max(ones, n - ones) + k)
let small = n - big
print(1 + big * (big + 1) / 2 + small * (small + 1) / 2)

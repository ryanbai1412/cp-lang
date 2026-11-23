fn f(c: Int) -> Int = c * (c + 1) / 2

@!multitest
let (n, k): (1..=2e5, 0..=2e5) = read()
let s: Str = read()
let ones = s.count(c => c == '1')
let best = [min(n, ones + k), max(0, ones - k)].map(c => 1 + f(c) + f(n - c)).max()!
print(best)

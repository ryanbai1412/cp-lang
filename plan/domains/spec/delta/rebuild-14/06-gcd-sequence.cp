// Codeforces 1980D GCD-sequence
// in: t; per case n, then n values a. out: per case YES if removing exactly one element makes the GCD sequence non-decreasing, else NO
@!multitest
let n = read<3..=2e5>()
let a = read<[n]1..=1e9>()

fn sorted_without(skip: Int) -> Bool:
    let b = [a[i] for i in 0..n if i != skip]
    [gcd(b[i], b[i + 1]) for i in 0..b.len - 1].is_sorted()

let g = [gcd(a[i], a[i + 1]) for i in 0..n - 1]
if let i = (0..n - 2).find(i => g[i] > g[i + 1]):
    print(yes([i, i + 1, i + 2].any(j => sorted_without(j))))
else:
    print(yes(true))

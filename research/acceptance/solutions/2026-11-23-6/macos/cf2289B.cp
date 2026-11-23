@!multitest
let (n, k): (1..=2e5, 1..=2e5) = read()
let a: [0..n]1..=2e5 = read()
let w = n - k + 1
print(sum(max(0, r.len - w + 1) for r in a.runs()))

@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let p = prefix(a)
let tot = p.prefix(n)
let below = any(p.sum(..i) * n < tot * i for i in 1..n)
let no_gap = all(p.sum(..i) * n == tot * i or p.sum(..i + 1) * n == tot * (i + 1) for i in 0..n)
print(if below or no_gap: "Yes" else: "No")

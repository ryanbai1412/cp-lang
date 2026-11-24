let n: 30..=500_000 = read()
domain I = 0..n
let a: [I]0..=1_000_000_000 = read()
let hi = a.max_k(30).sorted()
let lo = a.min_k(30).sorted()
print(hi[0], hi[29], lo[0], lo[29])

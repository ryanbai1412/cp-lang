// Codeforces 2229 B: orient each pair (a[i], b[i]) so a[i] <= b[i], then add
// max(a) to sum(b).
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()
let lo = [0..n] i => min(a[i], b[i])
let hi = [0..n] i => max(a[i], b[i])
print(sum(hi) + max(lo)!)

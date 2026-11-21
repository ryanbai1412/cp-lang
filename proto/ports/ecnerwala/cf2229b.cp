// Codeforces 2229 B: orient each pair (a[i], b[i]) so a[i] <= b[i], then add
// max(a) to sum(b).
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()
print(sum(max(x, y) for (x, y) in zip(a, b)) + max(min(x, y) for (x, y) in zip(a, b))!)

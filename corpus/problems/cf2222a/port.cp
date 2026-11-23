// Codeforces 2222 A — Check for 100
//
// Check whether any element of `a[0..n]` equals `100`.

@!multitest
let n: 1..=100 = read()
let a: [0..n]1..=10 ** 9 = read()
print(if a.any(x => x == 100): "Yes" else: "No")

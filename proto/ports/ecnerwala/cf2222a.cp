// Codeforces 2222 A: check whether any element of the array equals 100.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
print(if any(x == 100 for x in a): "Yes" else: "No")

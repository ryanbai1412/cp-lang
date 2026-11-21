// Codeforces 2152 A: each distinct value needs one rise and (except the last)
// one drop, giving 2 * distinct - 1 operations.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
print(2 * Set(a).len - 1)

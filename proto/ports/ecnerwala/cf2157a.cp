// Codeforces 2157 A: for each value v, either drop all freq[v] copies or keep
// exactly v of them.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var freq: [0..=n]Int = 0
for x in a: freq[x] += 1
print(sum(if freq[v] < v: freq[v] else: freq[v] - v for v in 0..=n))

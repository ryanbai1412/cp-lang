// expect: ok, output "45"
// rule: R02.10.2, R03.1.4
// input: "5\n"
// Optimized IR leaves out the domain and range checks the loop's facts prove (`i` in `0..n`
// indexing `[0..n]`, `n - i` in `1..=n`, `i + 1` in `0..=n`); the results are the same.
let n: 1..=10 = read()
var walk: List<1..=n> = []
for i in 0..n: walk.push(n - i)
var nxt: [0..n]0..=n = 0
for i in 0..n: nxt[i] = i + 1
var s = 0
for i in 0..n: s += walk[i] * 2 + nxt[i]
print(s)

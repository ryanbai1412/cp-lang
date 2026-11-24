// dmopc21c5p4, cf2187f1: a list or array whose range has a runtime bound (`1..=n`, `0..=n`) is
// stored in the 32 bits that bound's own range proves, as in the matched C++.
let n: 1..=4_000_000 = read()
var walk: List<1..=n> = []
for i in 0..n: walk.push(n - i)
var nxt: [0..n]0..=n = 0
for i in 0..n: nxt[i] = i + 1
var s = 0
for i in 0..n: s += walk[i] * 2 + nxt[i]
print(s)

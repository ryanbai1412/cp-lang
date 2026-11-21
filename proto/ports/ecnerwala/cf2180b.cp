// Codeforces 2180 B: greedily choose min(s + a, a + s) for each incoming string.
@!multitest
let n: Int = read()
var s = ""
for _ in 0..n:
    let a: Str = read()
    s = min(s + a, a + s)
print(s)

// Codeforces 2180 B: greedily choose min(s + a, a + s) for each incoming string.
@!multitest
let n: Int = read()
var s = ""
for _ in 0..n:
    let a: Str = read()
    let s1 = s + a
    let s2 = a + s
    s = if s1 < s2: s1 else: s2
print(s)

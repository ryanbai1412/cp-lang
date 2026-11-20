// Codeforces 2237 C: greedily replace running total with a[i] whenever a[i] >= cur.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var cur = a[0]
for i in 1..n:
    if a[i] >= cur: cur = a[i]
    else: cur += a[i]
print(cur)

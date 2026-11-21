// Codeforces 2237 C: greedily replace running total with a[i] whenever a[i] >= cur.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var cur = a[0]
for x in a[1..n]: cur = if x >= cur: x else: cur + x
print(cur)

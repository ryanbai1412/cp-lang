let n: 1..=100 = read()
domain V = 1..=n
let a: [V]Int = read()

var pre: [V.cuts]Int = 0
for v in V:
    pre[v.r] = pre[v.l] + a[v]
print(pre[V.last.r])

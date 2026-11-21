@!multitest
let n: Int = read()
var val: [0..=n](Int, Int) = (0, 0)
var nxt = 0
var a: [0..n]Int = 0
for i in 0..n:
    let b: Int = read()
    if val[b].0 == 0:
        nxt += 1
        val[b] = (nxt, b)
    a[i] = val[b].0
    val[b] = (val[b].0, val[b].1 - 1)
    if val[b].1 == 0:
        val[b] = (0, 0)
let good = all(val[i].0 == 0 for i in 0..=n)
if good:
    print(a)
else:
    print(-1)

@!multitest
let n: Int = read()
var a: List<Int> = [read::<Int>() - 1 for _ in 0..n]
domain V = 0..n
fn parity(vals: List<Int>) -> Int:
    var bit = Fenwick<V, Int>()
    var ans = 0
    var seen = 0
    for v in vals:
        ans ^= (seen - bit.prefix(V(v).r)) & 1
        bit.add(V(v), 1)
        seen += 1
    ans
var x: List<Int> = []
var y: List<Int> = []
for i in 0..n:
    if (i & 1) == 0: x.push(a[i])
    else: y.push(a[i])
let bad = (parity(x) ^ parity(y)) != 0
x.sort()
y.sort()
for i in 0..n:
    a[i] = if (i & 1) == 0: x[i / 2] else: y[i / 2]
if bad:
    a.swap(n - 3, n - 1)
print([v + 1 for v in a])

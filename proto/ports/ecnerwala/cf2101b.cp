@!multitest
let n: Int = read()
var a = [0..n] _ => read::<Int>() - 1
domain V = 0..n

fn parity(vals: List<Int>) -> Int:
    var bit = Fenwick<V, Int>()
    var ans = 0
    for (seen, v) in vals.items:
        ans ^= (seen - bit.prefix(V(v).r)) & 1
        bit.add(V(v), 1)
    ans

var x = [a[i] for i in 0..n by 2]
var y = [a[i] for i in 1..n by 2]
let bad = (parity(x) ^ parity(y)) != 0
x.sort()
y.sort()
for (i, v) in x.items: a[2 * i] = v
for (i, v) in y.items: a[2 * i + 1] = v
if bad: a.swap(n - 3, n - 1)
print([v + 1 for v in a])

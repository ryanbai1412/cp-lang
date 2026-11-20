@!multitest
let (n, q): (Int, Int) = read()
var a: List<Int> = [read::<Int>() - 1 for _ in 0..n]
var prv0: [0..n]Int = 0
var prv1: [0..n]Int = 0
for z in 0..2:
    var stk: List<(Int, Int)> = [(n, -1)]
    for i in 0..n:
        while stk.last!.0 < a[i]: let _ = stk.pop()
        if z == 0: prv0[i] = stk.last!.1 else: prv1[i] = stk.last!.1
        stk.push((a[i], i))
    a = [n - 1 - a[n - 1 - i] for i in 0..n]

var bad: [0..n]Int = n
for i in 0..n:
    let l = prv0[i]
    let r = n - 1 - prv1[n - 1 - i]
    if 0 <= l and r <= n - 1:
        bad[l] min= r
for i in (0..(n - 1)).rev: bad[i] min= bad[i + 1]
for _ in 0..q:
    let (l, r): (Int, Int) = read()
    print(if r - 1 >= bad[l - 1]: "NO" else: "YES")

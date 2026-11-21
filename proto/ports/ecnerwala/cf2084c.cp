@! multitest
let n: Int = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()
var a_to_b: [0..n]Int = 0
for i in 0..n:
    a_to_b[a[i] - 1] = b[i] - 1
var good = true
var fixed = 0
for i in 0..n:
    if a_to_b[i] == i:
        fixed += 1
    elif a_to_b[a_to_b[i]] != i:
        good = false
if fixed > 1:
    good = false
if not good:
    print(-1)
else:
    var inv_a: [0..n]Int = 0
    for i in 0..n:
        inv_a[a[i] - 1] = i
    var p: [0..n]Int = -1
    var cur = 0
    for i in 0..n:
        if p[i] != -1:
            continue
        if a[i] == b[i]:
            p[i] = n / 2
        else:
            let j = inv_a[b[i] - 1]
            p[i] = cur
            p[j] = n - 1 - cur
            cur += 1
    var ops: List<(Int, Int)> = []
    for i in 0..n:
        while p[i] != i:
            let j = p[i]
            ops.push((i + 1, j + 1))
            p.swap(i, j)
    print(ops.len)
    for (u, v) in ops:
        print(u, v)

@!multitest
let (n, k, q): (Int, Int, Int) = read()
var c1s: List<(Int, Int)> = []
var c2s: List<(Int, Int)> = []
for _ in 0..q:
    let (c, l, r): (Int, Int, Int) = read()
    if c == 1: c1s.push((l - 1, r))
    else: c2s.push((l - 1, r))
var a: [0..n]Int = k
for (l, r) in c2s:
    for i in l..r: a[i] = -1
for (l, r) in c1s:
    for i in l..r:
        if a[i] == -1: a[i] = k + 1
var cnt = 0
for i in 0..n:
    if a[i] == -1:
        a[i] = cnt % k
        cnt += 1
print(a)

@!multitest
let (n, k, q): (Int, Int, Int) = read()
var in_min: [0..n]Bool = false
var in_mex: [0..n]Bool = false

for _ in 0..q:
    let (c, l, r): (Int, Int, Int) = read()
    for i in l - 1..r:
        if c == 1: in_min[i] = true
        else: in_mex[i] = true

var a: [0..n]Int = k
var cnt = 0
for i in 0..n if in_mex[i]:
    if in_min[i]:
        a[i] = k + 1
    else:
        a[i] = cnt % k
        cnt += 1
print(a)

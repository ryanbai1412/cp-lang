let (n, q): (Int, Int) = read()
let a: [0..n]Int = read()
var p: [0..=n]Int = 0
for i in 0..n:
    p[i + 1] = p[i] + a[i]
for _ in 0..q:
    let (l, r): (Int, Int) = read()
    print(p[r] - p[l])

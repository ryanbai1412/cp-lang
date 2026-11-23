let (n, q): (1..=200000, 1..=200000) = read()
let a: [0..n]Int = read()
var d: [0..=n]Int = 0
for _ in 0..q:
    let k: 1..=n = read()
    print(a[n - k])
    let (l, r, x): (1..=n, 1..=n, Int) = read()
    d[l - 1] += x
    d[r] -= x
print(d)

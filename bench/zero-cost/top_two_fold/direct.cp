let (n, q): (1..=200_000, 1..=2_000_000) = read()
let a: [0..n]1..=1_000_000 = read()
var total = 0
for k in 0..q:
    let l = k * 7919 % (n - 8)
    var hi = 0
    var lo = 0
    for i in l..l + 8:
        let x = a[i]
        let h = max(hi, x)
        var second = 0
        if hi < h: second = max(second, hi)
        if lo < h: second = max(second, lo)
        if x < h: second = max(second, x)
        hi = h
        lo = second
    total += hi + lo
print(total)

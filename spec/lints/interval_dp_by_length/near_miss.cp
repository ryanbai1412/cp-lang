let n: 1..=3000 = read()
let a: [0..n]Int = read()
var best = 0
for l in 0..n:
    var s = 0
    for len in 1..=n - l:
        let r = l + len - 1
        s += a[r]
        best = max(best, s)
print(best)

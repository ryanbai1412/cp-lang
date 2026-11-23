let n: 1..=2e5 = read()
let a: [0..n]-1e9..=1e9 = read()
var best = a[0]
var cur = 0
for x in a:
    cur = max(cur + x, x)
    best max= cur
print(best)

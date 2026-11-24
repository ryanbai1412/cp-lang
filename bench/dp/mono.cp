use std.window.{Window}
let (n, k): (1..=5e6, 1..=5e6) = read()
let a: [0..n]0..=1e9 = read()
var w = Window(min)
var total = 0
for (i, x) in a.items:
    w.push(x)
    if i >= k: w.pop(a[i - k])
    total += w.all!
print(total)

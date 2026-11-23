use std.algebra.{desc}

@!multitest
let n: 2..=6 = read()
domain I = 1..=n
let a: [I]0..=6 = read()

var ws = [1 << x for x in a]
sort(mut ws)
debug("sorted", ws)

var x = 0
var y = 0
for w in ws:
    if x <= y: x += w
    else: y += w
debug("piles", x, y)
print(x * y)

@brute(main)
fn slow():
    var best = 0
    var total = 0
    for i in I: total += 1 << a[i]
    for mask in 0..(1 << n):
        var s = 0
        for i in I:
            if (mask >> i.to_idx) & 1 == 1: s += 1 << a[i]
        best max= s * (total - s)
    debug("piles", best, total)
    print(best)

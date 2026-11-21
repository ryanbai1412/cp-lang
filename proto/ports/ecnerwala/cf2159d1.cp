use std.math.{ceil_div}

@! multitest
let n: Int = read()
let a: [0..n]Int = read()

var stk: [0..=n](val: Int, dist: Int) = (-1, 0)
var top = 0

for i in 0..n:
    let x = a[i]
    while top > 0 and stk[top].val >= x: top -= 1
    var dist = 1 + stk[top].dist
    for l in 2..=min(dist, 3):
        let tgt = ceil_div(x, l)
        let lo = bsearch_last(0..=top, md => stk[md].val < tgt)!
        dist min= stk[lo].dist + l
    top += 1
    stk[top] = (x, dist)

print(stk[top].dist)

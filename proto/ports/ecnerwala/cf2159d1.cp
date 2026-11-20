use std.math.{ceil_div}

@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var stk: List<(Int, Int)> = [(-1, 0)]
for i in 0..n:
    let x = a[i]
    while stk.len > 1 and stk.last!.0 >= x:
        let _ = stk.pop()
    var dist = 1 + stk.last!.1
    for l in 2..=min(dist, 3):
        let tgt = ceil_div(x, l)
        var lo = 0
        var hi = stk.len
        while hi - lo > 1:
            let md = lo + (hi - lo) / 2
            if stk[md].0 >= tgt: hi = md
            else: lo = md
        dist min= stk[lo].1 + l
    stk.push((x, dist))
print(stk.last!.1)

use std.math.{ceil_div}

@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var stk: List<(Int, List<Int>)> = [(-1, [-1])]
var ans = 0
for i in 0..n:
    let x = a[i]
    while stk.len > 1 and stk.last!.0 >= x:
        let _ = stk.pop()
    var min_idx = [i] + stk.last!.1
    var l = 2
    while l < min_idx.len and l <= 3:
        let tgt = ceil_div(x, l)
        var lo = 0
        var hi = stk.len
        while hi - lo > 1:
            let md = lo + (hi - lo) / 2
            if stk[md].0 >= tgt: hi = md
            else: lo = md
        let prev = stk[lo].1
        for k in 0..prev.len:
            min_idx[l + k] min= prev[k]
            if min_idx[l + k] == -1:
                min_idx = min_idx[0..=(l + k)]
                break
        l += 1
    for pos in min_idx:
        ans += pos + 1
    stk.push((x, min_idx))
print(ans)

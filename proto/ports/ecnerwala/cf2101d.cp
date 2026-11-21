@!multitest
let n: Int = read()
domain I = 0..n
let a_in: [I]Int = read()
var a: [I]Int = 0
for i in I:
    a[i] = a_in[i] - 1
var bad_r: [I]Int = n
var bad_l: [I]Int = n
for dir in 0..2:
    var cur: [I]Int = n
    var stk_lo: List<(Int, Int)> = [(-1, -1)]
    var stk_hi: List<(Int, Int)> = [(n, -1)]
    for i in I:
        let v = if dir == 0: a[i] else: a[I(n - 1 - i.ord)]
        while let top = stk_lo.last(), v < top.0:
            let _ = stk_lo.pop()
        while let top = stk_hi.last(), v > top.0:
            let _ = stk_hi.pop()
        let lo_idx = stk_lo.last()!.1
        let hi_idx = stk_hi.last()!.1
        let b = min(lo_idx, hi_idx)
        if b >= 0:
            let bi = I(b)
            cur[bi] min= i.ord
        stk_lo.push((v, i.ord))
        stk_hi.push((v, i.ord))
    var j = n - 1
    while j > 0:
        let prev = I(j - 1)
        let curr = I(j)
        cur[prev] min= cur[curr]
        j -= 1
    if dir == 0:
        bad_r = cur
    else:
        bad_l = cur
var ans = 0
for i in I:
    let bl = n - 1 - bad_l[I(n - 1 - i.ord)]
    ans += (bad_r[i] - i.ord) * (i.ord - bl)
for k in 0..n - 1:
    let bl_next = n - 1 - bad_l[I(n - 1 - (k + 1))]
    ans -= (bad_r[I(k)] - (k + 1)) * (k - bl_next)
print(ans)

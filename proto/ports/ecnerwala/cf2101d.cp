@!multitest
let n: Int = read()
let a = [0..n] _ => read::<Int>() - 1

var bad_r: [0..n]Int = n
var bad_l: [0..n]Int = n
for dir in 0..2:
    var cur: [0..n]Int = n
    var stk_lo: List<(Int, Int)> = [(-1, -1)]
    var stk_hi: List<(Int, Int)> = [(n, -1)]
    for i in 0..n:
        let v = if dir == 0: a[i] else: a[n - 1 - i]
        while let top = stk_lo.last, v < top.0: stk_lo.pop()
        while let top = stk_hi.last, v > top.0: stk_hi.pop()
        let b = min(stk_lo.last!.1, stk_hi.last!.1)
        if b >= 0: cur[b] min= i
        stk_lo.push((v, i))
        stk_hi.push((v, i))
    for j in (1..n).rev: cur[j - 1] min= cur[j]
    if dir == 0: bad_r = cur
    else: bad_l = cur

var ans = 0
for i in 0..n:
    let bl = n - 1 - bad_l[n - 1 - i]
    ans += (bad_r[i] - i) * (i - bl)
for k in 0..n - 1:
    let bl_next = n - 1 - bad_l[n - 1 - (k + 1)]
    ans -= (bad_r[k] - (k + 1)) * (k - bl_next)
print(ans)

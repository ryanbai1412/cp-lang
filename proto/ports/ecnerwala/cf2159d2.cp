@!multitest
let n: Int = read()
let a: [0..n]Int = read()

var stk: [0..=n](val: Int, idx: List<Int>) = (-1, [-1])
var top = 0
var ans = 0

for i in 0..n:
    let x = a[i]
    while top > 0 and stk[top].val >= x: top -= 1
    var min_idx = [i] + stk[top].idx
    for l in 2..=3:
        if l >= min_idx.len: break
        let tgt = ceil_div(x, l)
        let lo = bsearch_last(0..=top, md => stk[md].val < tgt)!
        let prev = stk[lo].idx
        for k in 0..prev.len:
            min_idx[l + k] min= prev[k]
            if min_idx[l + k] == -1:
                min_idx = min_idx[0..=l + k]
                break
    ans += sum(pos + 1 for pos in min_idx)
    top += 1
    stk[top] = (x, min_idx)

print(ans)

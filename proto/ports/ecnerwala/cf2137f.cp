@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()

var stk: List<(val: Int, idx: Int)> = []
var ans = 0
for i in 0..n:
    let lo: ?Int = if a[i] == b[i]:
        i
    else:
        let need = max(a[i], b[i])
        if let j = bsearch_last(0..stk.len, m => stk[m].val >= need): stk[j].idx else: none
    if let l = lo: ans += (l + 1) * (n - i)
    while let top = stk.last, top.val <= a[i]: stk.pop()
    stk.push((a[i], i))

print(ans)

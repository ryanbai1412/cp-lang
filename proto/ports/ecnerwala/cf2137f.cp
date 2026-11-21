@!multitest
let n: Int = read()
let a: List<Int> = [read::<Int>() for _ in 0..n]
let b: List<Int> = [read::<Int>() for _ in 0..n]
var stk: List<(Int, Int)> = []
var ans = 0
for i in 0..n:
    let lo = if a[i] == b[i]:
        i
    else:
        let need = max(a[i], b[i])
        if let j = bsearch_last(0..stk.len, m => stk[m].0 >= need):
            stk[j].1
        else:
            -1
    ans += (lo + 1) * (n - i)
    while let top = stk.last():
        if top.0 > a[i]: break
        stk.pop()
    stk.push((a[i], i))
print(ans)

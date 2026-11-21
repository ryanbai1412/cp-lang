@!multitest
let n: Int = read()
let _k: Int = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()
var pref = 0
var min_val = 0
var min_pos = 0
for i in 0..n:
    pref += a[i] - b[i]
    if pref < min_val:
        min_val = pref
        min_pos = i + 1
var stk: List<(Int, Int)> = []
var ans = 0
for i in 0..n:
    let idx = (min_pos + i) % n
    stk.push((a[idx], i))
    var rem = b[idx]
    while let top = stk.last(), rem > 0:
        let t = min(top.0, rem)
        ans max= i + 1 - top.1
        let _ = stk.pop()
        if top.0 > t:
            stk.push((top.0 - t, top.1))
        rem -= t
print(ans)

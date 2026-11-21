@!multitest
let (n, _k): (Int, Int) = read()
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
    while let (c, pos) = stk.last, rem > 0:
        let t = min(c, rem)
        ans max= i + 1 - pos
        stk.pop()
        if c > t: stk.push((c - t, pos))
        rem -= t
print(ans)

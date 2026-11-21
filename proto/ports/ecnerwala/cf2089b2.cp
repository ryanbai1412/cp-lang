@!multitest
let (n, k_in): (Int, Int) = read()
var k = k_in
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
var cnt_dist: [0..=n]Int = 0
for i in 0..n:
    let idx = (min_pos + i) % n
    stk.push((a[idx], i))
    var rem = b[idx]
    while let (c, pos) = stk.last, rem > 0:
        let t = min(c, rem)
        cnt_dist[i + 1 - pos] += t
        stk.pop()
        if c > t: stk.push((c - t, pos))
        rem -= t

var ans = n
while ans > 0:
    if cnt_dist[ans] == 0:
        ans -= 1
    elif k > 0:
        let v = min(k, cnt_dist[ans])
        cnt_dist[ans] -= v
        k -= v
    else:
        break
print(ans)

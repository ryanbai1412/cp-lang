@!multitest
let (n, k0): (Int, Int) = read()
let a: List<Int> = [read::<Int>() for _ in 0..n]
let k = k0 - 1
if k == 0:
    print("YES")
    return
var sorted = a
sorted.sort()
let thresh = sorted[k - 1]
var freq: List<Int> = []
var vals: List<Int> = []
var cur_f = 0
for v in a:
    if v < thresh:
        freq.push(cur_f)
        cur_f = 0
        vals.push(v)
    elif v == thresh:
        cur_f += 1
freq.push(cur_f)
var good = all(vals[i] == vals[vals.len - 1 - i] for i in 0..vals.len)
var cnt = vals.len
var l = 0
var r = freq.len - 1
while l <= r:
    if l == r: cnt += freq[l]
    else: cnt += 2 * min(freq[l], freq[r])
    l += 1
    r -= 1
if cnt < k: good = false
print(if good: "YES" else: "NO")

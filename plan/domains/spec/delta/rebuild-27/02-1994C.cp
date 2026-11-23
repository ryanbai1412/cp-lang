// CF 1994C: count segments whose final g is non-zero.
// nxt[l] = first r where the sum from l passes x (two pointers); then g restarts at r + 1.
@!multitest
let (n, x): (1..=2e5, 1..=1e9) = read()
let a = read<[n]1..=1e9>()

var nxt: [n]?Int = None
var r = 0
var sum = 0
for l in 0..n:
    if r < l:
        r = l
        sum = 0
    while r < n and sum + a[r] <= x:
        sum += a[r]
        r += 1
    if r < n: nxt[l] = Some(r)
    if r > l: sum -= a[l]

var good: [0..=n]Int = 0
for l in (0..n).rev:
    good[l] = if let Some(k) = nxt[l]: k - l + good[k + 1] else: n - l
print(good.sum())

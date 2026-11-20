// Codeforces 2201 A2: start from total subarray lengths and subtract (p + 1) * (N - i)
// whenever element i unlinks against its live predecessor p with value a - 1.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let vals = sorted([(-a[i], i) for i in 0..n])

var prv: [0..n]?Int = none
var nxt: [0..n]?Int = none
for i in 0..n:
    if i > 0: prv[i] = i - 1
    if i + 1 < n: nxt[i] = i + 1

var ans = n * (n + 1) * (n + 2) / 6
for (neg_v, i) in vals:
    let v = -neg_v
    if let p = prv[i]:
        if a[p] == v - 1:
            ans -= (p + 1) * (n - i)
            let nx = nxt[i]
            nxt[p] = nx
            if let q = nx: prv[q] = p
print(ans)

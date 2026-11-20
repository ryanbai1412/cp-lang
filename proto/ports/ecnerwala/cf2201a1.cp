// Codeforces 2201 A1: process elements in descending value (then ascending index)
// order and unlink any element whose live predecessor has value a - 1.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let vals = sorted([(-a[i], i) for i in 0..n])

var prv: [0..n]?Int = none
var nxt: [0..n]?Int = none
for i in 0..n:
    if i > 0: prv[i] = i - 1
    if i + 1 < n: nxt[i] = i + 1

var ans = n
for (neg_v, i) in vals:
    let v = -neg_v
    if let p = prv[i]:
        if a[p] == v - 1:
            ans -= 1
            let nx = nxt[i]
            nxt[p] = nx
            if let q = nx: prv[q] = p
print(ans)

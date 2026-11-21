// Codeforces 2201 A1: process elements in descending value (then ascending index)
// order and unlink any element whose live predecessor has value a - 1.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()

var prv: [0..n]?Int = [0..n] i => if i > 0: i - 1 else: none
var nxt: [0..n]?Int = [0..n] i => if i + 1 < n: i + 1 else: none
var ans = n
for i in (0..n).sorted(by: i => (-a[i], i)):
    if let p = prv[i], a[p] == a[i] - 1:
        ans -= 1
        let nx = nxt[i]
        nxt[p] = nx
        if let q = nx: prv[q] = p
print(ans)

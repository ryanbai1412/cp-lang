// Codeforces 2201 A2: start from total subarray lengths and subtract (p + 1) * (N - i)
// whenever element i unlinks against its live predecessor p with value a - 1.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()

var prv: [0..n]?Int = [0..n] i => if i > 0: i - 1 else: none
var nxt: [0..n]?Int = [0..n] i => if i + 1 < n: i + 1 else: none
var ans = n * (n + 1) * (n + 2) / 6
for i in sorted(0..n, by: i => (-a[i], i)):
    if let p = prv[i], a[p] == a[i] - 1:
        ans -= (p + 1) * (n - i)
        let nx = nxt[i]
        nxt[p] = nx
        if let q = nx: prv[q] = p
print(ans)

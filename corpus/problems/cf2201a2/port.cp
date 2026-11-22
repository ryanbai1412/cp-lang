// Codeforces 2201 A2 — Lost Civilization (Hard Version)
//
// Sum the minimum remaining size over all subarrays `a[l..=r]`. Without any
// absorptions, the sum of all subarray lengths is
// `sum_{len=1..n} len * (n - len + 1) = n * (n + 1) * (n + 2) / 6`.
//
// Processing indices in descending `a[i]` (ties left-to-right) on the live
// doubly linked list `(prv, nxt)`, element `i` is absorbed by its nearest
// surviving predecessor `p = prv[i]` with `a[p] == a[i] - 1`. That absorption
// occurs in every subarray `[l, r]` containing both `p` and `i` (`l <= p` and
// `r >= i`), of which there are `(p + 1) * (n - i)`, so unlinking `i`
// subtracts `(p + 1) * (n - i)` from the total.

@!multitest
let n: 1..=300_000 = read()
let a: [0..n]1..=10 ** 9 = read()

var prv: [0..n]?Int = [0..n] i => if i > 0: i - 1 else: none
var nxt: [0..n]?Int = [0..n] i => if i + 1 < n: i + 1 else: none
var ans = n * (n + 1) * (n + 2) / 6
for i in (0..n).sorted(by: i => (-a[i], i)):
    if let p = prv[i] and a[p] == a[i] - 1:
        ans -= (p + 1) * (n - i)
        let nx = nxt[i]
        nxt[p] = nx
        if let q = nx: prv[q] = p
print(ans)

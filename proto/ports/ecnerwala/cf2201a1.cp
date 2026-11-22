// Codeforces 2201 A1 — Lost Civilization (Easy Version)
//
// An element `a[i]` can be absorbed by a preceding element of value `a[i] - 1`
// once every larger element between them has been absorbed. Processing indices
// in descending order of `a[i]` (ties left-to-right) ensures that when `i` is
// visited, all elements `> a[i]` have already been resolved, and any earlier
// equal element `a[j] == a[i]` has already been removed if it could be; thus
// `i` is absorbable iff its live predecessor `p = prv[i]` has `a[p] == a[i] - 1`.
// Unlinking `i` from the doubly linked list `(prv, nxt)` decreases the
// remaining count by 1.

@!multitest
let n: 1..=300_000 = read()
let a: [0..n]1..=10 ** 9 = read()

var prv: [0..n]?Int = [0..n] i => if i > 0: i - 1 else: none
var nxt: [0..n]?Int = [0..n] i => if i + 1 < n: i + 1 else: none
var ans = n
for i in (0..n).sorted(by: i => (-a[i], i)):
    if let p = prv[i] and a[p] == a[i] - 1:
        ans -= 1
        let nx = nxt[i]
        nxt[p] = nx
        if let q = nx: prv[q] = p
print(ans)

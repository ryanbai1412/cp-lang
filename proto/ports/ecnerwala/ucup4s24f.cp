// UCup 4 Stage 24 (Wulin) F — Residue-Covering Minimum Cost Cuts
//
// Scan prefix sums `pref.prefix(i) % r` from `n` down to `1`, tracking each
// residue's earliest seen index `last_occ[rem]` and the farthest among all `r`
// residues via a lazy `MaxHeap` (`first_full`, or `n + 1` until all `r`
// residues appear).
//
// Choosing position `i` (cost `c[i - 1]`) requires the next chosen position to
// lie in `i + 1..=first_full` so no gap contains all `r` residues; a monotone
// deque (`MonoDeque<Int>.new(min)`) maintains `min(dp[i + 1..=first_full])` in
// O(1) amortized time. Finally, the first chosen position must lie in
// `1..=first_0` where `first_0 = last_occ.get(0) ?? (n + 1)`.

use std.window.{MonoDeque}

@!multitest
let (n, r): (1..=500_000, 1..=500_000) = read()
let a: [0..n]Int = read()
let c: [0..n]Int = read()
let pref = prefix(a)

var last_occ: Map<Int, Int> = Map()
var occ_heap = MaxHeap<(Int, Int)>()
var dp: [0..=n + 1]Int = 0
var window = MonoDeque<Int>.new(min)
window.push(n + 1, 0)

for i in (1..=n).rev:
    let rem = pref.prefix(i) % r
    last_occ[rem] = i
    occ_heap.push((i, rem))

    var first_full = n + 1
    if last_occ.len == r:
        while let (pos, val) = occ_heap.peek and last_occ[val] != pos:
            occ_heap.pop()
        first_full = occ_heap.peek!.0

    window.drop_after(first_full)
    dp[i] = c[i - 1] + window.best!
    window.push(i, dp[i])

let first_0 = last_occ.get(0) ?? (n + 1)
print(min(dp[i] for i in 1..=first_0)!)

// UCup 4 Stage 24 (Wulin) F: scan prefix sums mod R right-to-left, maintaining
// the farthest last-occurrence among all R residues via a lazy max-heap and
// taking the next DP state as the minimum over a sliding window.
use std.window.{MonoDeque}
@!multitest
let (n, r): (Int, Int) = read()
let a: [0..n]Int = read()
let c: [0..n]Int = read()
let pref = prefix1(a)

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
        while let (pos, val) = occ_heap.peek, last_occ[val] != pos:
            occ_heap.pop()
        first_full = occ_heap.peek!.0

    window.drop_after(first_full)
    dp[i] = c[i - 1] + window.best!
    window.push(i, dp[i])

let first_0 = last_occ.get(0) ?? (n + 1)
print(min(dp[i] for i in 1..=first_0)!)

// UCup 4 Stage 24 (Wulin) F: scan prefix sums mod R right-to-left, maintaining
// the farthest last-occurrence among all R residues via a lazy max-heap and
// querying the minimum valid next DP state with a monotonic deque.
@! multitest
let (n, r): (Int, Int) = read()
let a: [0..n]Int = read()
let c: [0..n]Int = read()

var pref: [0..=n]Int = 0
for i in 0..n: pref[i + 1] = (pref[i] + a[i]) % r

var last_occ: Map<Int, Int> = Map()
var occ_heap = MaxHeap<(Int, Int)>()
var dp: [0..=n + 1]Int = 0
var dq: Deque<(Int, Int)> = Deque()
dq.push((0, n + 1))

for i in (1..=n).rev:
    let rem = pref[i]
    last_occ[rem] = i
    occ_heap.push((i, rem))

    var first_full = n + 1
    if last_occ.len == r:
        while let (pos, val) = occ_heap.peek:
            if last_occ[val] == pos:
                first_full = pos
                break
            occ_heap.pop()

    while let (_, idx) = dq.first, idx > first_full:
        dq.pop_front()
    dp[i] = c[i - 1] + dq.first!.0
    while let (val, _) = dq.last, dp[i] <= val:
        dq.pop()
    dq.push((dp[i], i))

let first_0 = last_occ.get(0) ?? (n + 1)
print(min(dp[i] for i in 1..=first_0)!)

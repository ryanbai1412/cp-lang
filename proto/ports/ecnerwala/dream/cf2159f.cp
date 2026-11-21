// Codeforces 2159 F: Grand Finale: Snakes (dream).
// An `n x n` grid holds a permutation of `1..=n^2`. A snake of length `l`
// (`1 <= l <= n`) walks along the `2n - 1` steps `t in 0..=2n - 2`; `query(l, t)`
// returns the maximum value on the snake at time `t`.
// Each grid value `v` enters the snake's sweep at diagonal index
// `diag[v] = (n - 1) - j` on row 0 and `(n - 1) + i + j` on rows `i >= 1`.
// Over any stride `[t0, t1]` of length `<= l`, the query sequence is bitonic
// (decreasing while `diag[v] < t0 + n`, then increasing), so binary search
// splits `[t0, t1]` into monotone runs `Seg(val, len, cur, end)` whose values
// increase as `cur` steps toward `end`. A `MinHeap` over these monotone runs
// yields the `m` smallest query values in order.
@!interactive

// A monotone run of query values for snake length `len`, starting at time `cur`
// (value `val`) and stepping by `(end - cur).sign` toward exclusive `end`.
struct Seg(val: Int, len: Int, cur: Int, end: Int)

fn query(len: Int, t: Int) -> Int:
    print("?", len, t + 1)
    read::<Int>() - 1

@!multitest
let n: 1..=500 = read()
let m: 1..=250_000 = read()
let max_t = 2 * n - 2
let a: [0..n, 0..n]1..=250_000 = read()

// Diagonal arrival index of each 0-based grid value `v`.
var diag: [0..n * n]Int = 0
for i in 0..n:
    for j in 0..n:
        diag[a[i, j] - 1] = if i == 0: n - 1 - j else: n - 1 + i + j

var heap = MinHeap<(Int, Seg)>()
fn push_seg(val: Int, len: Int, cur: Int, end: Int):
    heap.push((val, Seg(val, len, cur, end)))

for len in 1..=n:
    var v0 = max(a[0, j] - 1 for j in 0..len)
    var t0 = 0
    while t0 < max_t:
        push_seg(v0, len, t0, t0 + 1)
        let t1 = min(t0 + len, max_t)
        let v1 = query(len, t1)
        // Bisect the bitonic valley in `(t0, t1)`, pushing each monotone half.
        var (lo, hi) = (t0, t1)
        while hi - lo > 1:
            let mid = (lo + hi) / 2
            let v_mid = query(len, mid)
            if diag[v_mid] < t0 + n:
                push_seg(v_mid, len, mid, lo)
                lo = mid
            else:
                push_seg(v_mid, len, mid, hi)
                hi = mid
        (t0, v0) = (t1, v1)
    push_seg(v0, len, t0, t0 + 1)

var ans: List<Int> = []
repeat m:
    let (_, seg) = heap.pop()!
    ans.push(seg.val + 1)
    let next_t = seg.cur + (seg.end - seg.cur).sign
    if next_t != seg.end:
        push_seg(query(seg.len, next_t), seg.len, next_t, seg.end)
print("!", ans)

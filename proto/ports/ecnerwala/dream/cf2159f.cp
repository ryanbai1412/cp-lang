// Codeforces 2159 F: Grand Finale: Snakes (interactive).
// An `n x n` grid holds a permutation of `Val = 1..=n^2`. A snake of length
// `len` (`1 <= len <= n`) advances across `Time = 1..=2n - 1`; `query(len, t)`
// returns the maximum value currently on the snake.
//
// Arrival diagonal: at `t = 1` the snake covers row 0 from column `n - 1` down
// to `0`, so cell `(0, j)` enters at `diag = (n - 1) - j`; cells on rows
// `i >= 1` enter along anti-diagonals `i + j` at `diag = (n - 1) + i + j`.
// Over any stride `[t0, t1]` of length `<= len`, cells leaving the snake have
// `diag < t0.ord + n` while newly entering cells have `diag >= t0.ord + n`, so
// `query(len, t)` is bitonic (decreasing until the old max leaves, then
// increasing as new maxima enter).
//
// Partitioning each stride: binary search maintains `(lo, hi)` straddling the
// valley. Probing `mid` lands on either the decreasing half (emitting the
// strictly increasing run `mid` down to `lo + 1`, then raising `lo = mid`) or
// the increasing half (emitting `mid` up to `hi - 1`, then lowering `hi = mid`).
// Together with the endpoints `t0` and `t1`, the pushed runs partition `[t0, t1]`
// into disjoint monotone runs — emitting each `(len, t)` once — in
// `O(sum_{len=1}^n (n / len) log(len) + m) = O(n log^2 n + m)` queries.

@!interactive

@!multitest
let n: 1..=500 = read()
let m: 1..=n * n = read()
domain Val = 1..=n * n
domain Time = 1..=2 * n - 1

// Monotone run starting at `cur` (value `val`) stepping by `dir` (-1 or +1)
// through `end` inclusive.
struct Seg(val: Val, len: Int, cur: Time, end: Time, dir: Int)

fn query(len: Int, t: Time) -> Val:
    print("?", len, t)
    read<Val>()

let a: [0..n, 0..n]Val = read()
var diag: [Val]Int = 0
for i in 0..n:
    for j in 0..n:
        diag[a[i, j]] = if i == 0: n - 1 - j else: n - 1 + i + j

var heap = MinHeap<Seg>(by: s => s.val)

for len in 1..=n:
    var v0 = max(a[0, j] for j in 0..len)!
    var t0 = Time.first!
    while t0 < Time.last!:
        heap.push(Seg(v0, len, t0, t0, 1))
        let t1 = t0.next(len) ?? Time.last!
        let v1 = query(len, t1)
        var (lo, hi) = (t0, t1)
        while hi - lo > 1:
            let mid = (lo..=hi).mid.r!
            let v_mid = query(len, mid)
            if diag[v_mid] < t0.ord + n:
                heap.push(Seg(v_mid, len, mid, lo.next!, -1))
                lo = mid
            else:
                heap.push(Seg(v_mid, len, mid, hi.prev!, 1))
                hi = mid
        (t0, v0) = (t1, v1)
    heap.push(Seg(v0, len, t0, t0, 1))

var ans: List<Val> = []
repeat m:
    let seg = heap.pop()!
    ans.push(seg.val)
    if seg.cur != seg.end:
        let next_t = (seg.cur + seg.dir)!
        heap.push(Seg(query(seg.len, next_t), seg.len, next_t, seg.end, seg.dir))
print("!", ans)

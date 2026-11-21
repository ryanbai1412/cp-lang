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
// increasing as new maxima enter). Binary search splits each stride into
// monotone runs `Seg(val, len, cur, end)` stepping toward `end`; a `MinHeap`
// merges all runs to emit the `m` smallest answers.

@!interactive

@!multitest
let n: 1..=500 = read()
let m: 1..=250_000 = read()
domain Val = 1..=n * n
domain Time = 1..=2 * n - 1

struct Seg(val: Val, len: Int, cur: Time, end: Int)

fn query(len: Int, t: Time) -> Val:
    print("?", len, t)
    read<Val>()

let a: [0..n, 0..n]Val = read()
var diag: [Val]Int = 0
for i in 0..n:
    for j in 0..n:
        diag[a[i, j]] = if i == 0: n - 1 - j else: n - 1 + i + j

var heap = MinHeap<(Val, Seg)>()
fn push_seg(val: Val, len: Int, cur: Time, end: Int):
    heap.push((val, Seg(val, len, cur, end)))

for len in 1..=n:
    var v0 = max(a[0, j] for j in 0..len)
    var t0 = Time.first
    while t0 < Time.last:
        push_seg(v0, len, t0, t0.ord + 1)
        let t1 = Time(min(t0.ord + 1 + len, 2 * n - 1))
        let v1 = query(len, t1)
        var (lo, hi) = (t0, t1)
        while hi.ord - lo.ord > 1:
            let mid = Time((lo.ord + hi.ord) / 2 + 1)
            let v_mid = query(len, mid)
            if diag[v_mid] < t0.ord + n:
                push_seg(v_mid, len, mid, lo.ord)
                lo = mid
            else:
                push_seg(v_mid, len, mid, hi.ord)
                hi = mid
        (t0, v0) = (t1, v1)
    push_seg(v0, len, t0, t0.ord + 1)

var ans: List<Val> = []
repeat m:
    let (_, seg) = heap.pop()!
    ans.push(seg.val)
    let next_ord = seg.cur.ord + (seg.end - seg.cur.ord).sign
    if next_ord != seg.end:
        let next_t = Time(next_ord + 1)
        push_seg(query(seg.len, next_t), seg.len, next_t, seg.end)
print("!", ans)

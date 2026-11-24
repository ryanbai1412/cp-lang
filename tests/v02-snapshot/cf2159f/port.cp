// Source: Codeforces 2159F, Grand Finale: Snakes, https://codeforces.com/contest/2159/problem/F
// Problem: an n x n grid (n <= 500) holds a permutation of 1..=n^2; hidden snakes of length
//   l = 1..=n start on row 1 and move down/right to (n, n) by second 2n - 1. Query f(l, T), the
//   max cell under snake l at second T, at most 120n + m times; print the m smallest f values.
//   Interactive multitest, sum n <= 500, sum m <= 5e4.
// Solution: order cells by when a snake reaches them (row 1 right to left, then by i + j);
//   snake l at second T covers the last l cells reached, so over l seconds f first falls, then
//   rises. Per l, query each stride's ends and binary-search the valley, pushing each probed
//   value as the head of a monotone run into a MinHeap. Pop the m smallest, querying a run's
//   next value only when its head pops: O(n log^2 n + m) queries.

@!interactive

@!multitest
let n: 1..=500 = read()
let m: 1..=250_000 = read()
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

var heap = MinHeap<(Val, Seg)>()
fn push_seg(val: Val, len: Int, cur: Time, end: Time, dir: Int):
    heap.push((val, Seg(val, len, cur, end, dir)))

for len in 1..=n:
    var v0 = (0..len).map(j => a[0, j]).max()!
    var t0 = Time.first!
    while t0 < Time.last!:
        push_seg(v0, len, t0, t0, 1)
        let t1 = Time.nth(min(t0.ord + len, Time.count - 1))
        let v1 = query(len, t1)
        var (lo, hi) = (t0, t1)
        while hi - lo > 1:
            let mid = Time.nth((lo.ord + hi.ord) / 2)
            let v_mid = query(len, mid)
            if diag[v_mid] < t0.ord + n:
                push_seg(v_mid, len, mid, lo.next!, -1)
                lo = mid
            else:
                push_seg(v_mid, len, mid, hi.prev!, 1)
                hi = mid
        (t0, v0) = (t1, v1)
    push_seg(v0, len, t0, t0, 1)

var ans: List<Val> = []
repeat m:
    let (_, seg) = heap.pop()!
    ans.push(seg.val)
    if seg.cur != seg.end:
        let next_t = seg.cur.offset(seg.dir)!
        push_seg(query(seg.len, next_t), seg.len, next_t, seg.end, seg.dir)
print("!", ans)

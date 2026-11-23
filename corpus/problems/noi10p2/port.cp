// Source: NOI 2010 P2 (Super Piano), https://dmoj.ca/problem/noi10p2
// Problem: n <= 5e5 notes of beauty |A_i| <= 1000 and L <= R. Choose k <= 5e5 distinct chords
//   (segments of L to R consecutive notes; enough exist) to maximise their total beauty, the
//   sum of their notes; print it.
// Solution: a segment from s is pre[e] - pre[s - 1] for e in a window, so keep in a max-heap,
//   per start, a window of ends with its best end from a SparseTable of prefix-sum argmaxes.
//   Taking the top k times splits its window around the end used into two candidates.
//   O((n + k) log n).
// original: corpus/submissions/5670502.cpp

let (n, k, lo_len, hi_len): (Int, Int, Int, Int) = read()
domain I = 1..=n
domain B = 0..=n
let a: [I]Int = read()

var pre: [B]Int = [B] _ => 0
for i in I:
    let idx = i.to_idx
    pre[B(idx + 1)] = pre[B(idx)] + a[i]

struct Pair:
    val: Int
    pos: Int

let rmq = SparseTable<B, Pair>(
    (x, y) => if x.val >= y.val: x else: y,
    [B] b => Pair(pre[b], b.to_idx),
)

struct Cand:
    score: Int
    start: Int
    lo: Int
    hi: Int
    best: Int

impl Ord for Cand:
    fn lt(a, b) = a.score < b.score

fn make_cand(start: Int, lo: Int, hi: Int) -> Cand:
    let mx = rmq.fold(B(lo)..=B(hi))
    Cand(mx.val - pre[B(start - 1)], start, lo, hi, mx.pos)

var pq = MaxHeap<Cand>()
for start in 1..=n:
    let lo = start + lo_len - 1
    if lo <= n:
        let hi = min(n, start + hi_len - 1)
        pq.push(make_cand(start, lo, hi))

var total = 0
for _ in 0..k:
    let c = pq.pop()!
    total += c.score
    if c.lo < c.best:
        pq.push(make_cand(c.start, c.lo, c.best - 1))
    if c.best < c.hi:
        pq.push(make_cand(c.start, c.best + 1, c.hi))
print(total)

// IOI 2011 P5 — Dancing Elephants: online minimum unit-length-`l` camera cover
// over `n` moving elephants via sqrt-decomposition buckets.
//
// Partition the sorted elephant coordinates into buckets of size `B = max(2, isqrt(n))`
// (`B >= 2` avoids rebuilding after every single update when `n = 1`). Every `B`
// updates we re-partition all `n` positions, so each bucket stays `O(B)` in size.
//
// Inside each bucket, a right-to-left two-pointer pass precomputes for every
// index `i` the greedy jump `Jump(end, count)` needed to cover `xs[i..]` with
// cameras of length `l`: placing a camera at `[xs[i], xs[i] + l]` jumps to the
// first `nxt` with `xs[nxt] > xs[i] + l` (or leaves the bucket in 1 camera).
//
// Each move updates at most two buckets in `O(B)`, and each query folds across
// the `O(n / B)` buckets carrying `cover: ?Int`, jumping to the first uncovered
// elephant in each bucket by binary search in `O(log B)` — `O(q sqrt(n) log n)` total.

struct Jump(end: Int, count: Int)

struct Bucket:
    xs: List<Int>
    jumps: List<Jump>

    fn rebuild(mut self, span: Int):
        let k = self.xs.len
        self.jumps = List.filled(k, Jump(0, 0))
        var j = k - 1
        for i in (0..k).rev:
            while j > i and self.xs[j] > self.xs[i] + span:
                j -= 1
            let nxt = j + 1
            self.jumps[i] = if nxt >= k:
                Jump(self.xs[i] + span, 1)
            else:
                Jump(self.jumps[nxt].end, self.jumps[nxt].count + 1)

    fn first_ge(self, x: Int) -> Int = bsearch_first(
        0..self.xs.len,
        i => self.xs[i] >= x,
    ) ?? self.xs.len
    fn first_gt(self, lim: Int) -> Int = bsearch_first(
        0..self.xs.len,
        i => self.xs[i] > lim,
    ) ?? self.xs.len

    fn remove(mut self, x: Int, span: Int):
        let _ = self.xs.remove(self.first_ge(x))
        self.rebuild(span)

    fn insert(mut self, x: Int, span: Int):
        self.xs.insert(self.first_ge(x), x)
        self.rebuild(span)

fn make_bucket(xs: List<Int>, span: Int) -> Bucket:
    var b = Bucket(xs, [])
    b.rebuild(span)
    b

fn bucket_for(buckets: List<Bucket>, x: Int) -> Int = first(
    i for i in 0..buckets.len if not buckets[i].xs.is_empty and x <= buckets[i].xs.last!
) ?? (buckets.len - 1)

let n: 1..=150_000 = read()
let l: 0..=1_000_000_000 = read()
let q: 1..=150_000 = read()

domain E = 0..n
var pos: [E]Int = read()
let b_size = max(2, isqrt(n))

fn rebuild_all() -> List<Bucket>:
    let all = [pos[e] for e in E].sorted()
    [make_bucket(all[i..min(i + b_size, n)], l) for i in (0..n).step(b_size)]

var buckets = rebuild_all()
var ops = 0

repeat q:
    let (e, y): (E, Int) = read()
    let old_x = pos[e]
    pos[e] = y

    buckets[bucket_for(buckets, old_x)].remove(old_x, l)
    buckets[bucket_for(buckets, y)].insert(y, l)

    ops += 1
    if ops >= b_size:
        ops = 0
        buckets = rebuild_all()

    var ans = 0
    var cover: ?Int = none
    for b in buckets:
        if b.xs.is_empty: continue
        let start = if let c = cover: b.first_gt(c) else: 0
        if start < b.xs.len:
            ans += b.jumps[start].count
            cover = some(b.jumps[start].end)
    print(ans)

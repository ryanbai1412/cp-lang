// CCO '16 P5 - Zombie Apocalypse (DMOJ: cco16p5, 15pt, TL 1.0s, ML 1024MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// N x M grid (N, M <= 1e9), K <= 2000 initial zombies at (r_i, c_i), time Q <= 1e9.
// Zombies spread to all 8 neighbors each second (Chebyshev radius Q).
// Output the number of cells first infected at exact second Q:
//   union_area(Q) - union_area(Q - 1).
//
// Classic Klee's Measure Problem: 2D coordinate-compressed sweep-line with a
// tag-free Cover-Count Segment Tree.
let (n, m): (1..=1e9, 1..=1e9) = read()
let k: 1..=2000 = read()
domain Z = 0..k
struct Pt(r: 1..=n, c: 1..=m)
let zombies: [Z]Pt = read()
let q: 0..=1e9 = read()

struct SweepEvent(x: Int, y1: Int, y2: Int, delta: Int)

// WISH A23: Cover-Count Segment Tree structurally breaks `LazySegTree<D, T, F>`!
// Updates are paired `+1`/`-1` over ranges with NO pushdown: a node's covered
// length is `if cnt[p] > 0: total_len[p] else if leaf: 0 else: cov[2p] + cov[2p+1]`.
// Because `apply(-1, x)` cannot know children's `cov` from `x` alone when `cnt`
// drops from 1 to 0, it is NOT an `Action<T, F>` and forces a hand-rolled
// recursive segment tree with raw `2*p` / `2*p + 1` indexing.
struct CoverTree:
    ys: List<Int>
    cnt: List<Int>
    cov: List<Int>

    pub fn new(ys: List<Int>) -> CoverTree:
        let segs = max(1, ys.len - 1)
        CoverTree(ys, List.filled(4 * segs, 0), List.filled(4 * segs, 0))

    fn update_at(mut self, p: Int, lo: Int, hi: Int, ql: Int, qr: Int, delta: Int):
        if qr <= lo or hi <= ql: return
        if ql <= lo and hi <= qr:
            self.cnt[p] += delta
        else:
            let mid = (lo + hi) / 2
            self.update_at(2 * p, lo, mid, ql, qr, delta)
            self.update_at(2 * p + 1, mid, hi, ql, qr, delta)
        if self.cnt[p] > 0:
            self.cov[p] = self.ys[hi] - self.ys[lo]
        elif lo + 1 == hi:
            self.cov[p] = 0
        else:
            self.cov[p] = self.cov[2 * p] + self.cov[2 * p + 1]

    pub fn add(mut self, ql: Int, qr: Int, delta: Int):
        if ql < qr:
            self.update_at(1, 0, self.ys.len - 1, ql, qr, delta)

    pub @get fn covered(self) -> Int = self.cov[1]

fn union_area(rad: Int) -> Int:
    if rad < 0: return 0
    var ys: List<Int> = []
    var evs: List<SweepEvent> = []
    for z in Z:
        let x1 = max(1, zombies[z].r - rad)
        let x2 = min(n, zombies[z].r + rad) + 1
        let y1 = max(1, zombies[z].c - rad)
        let y2 = min(m, zombies[z].c + rad) + 1
        ys.push(y1)
        ys.push(y2)
        evs.push(SweepEvent(x1, y1, y2, 1))
        evs.push(SweepEvent(x2, y1, y2, -1))

    ys.sort()
    var uniq: List<Int> = []
    for y in ys:
        if uniq.is_empty or uniq.last! != y:
            uniq.push(y)

    fn y_id(y: Int) -> Int:
        var (lo, hi) = (0, uniq.len)
        while lo < hi:
            let mid = (lo + hi) / 2
            if uniq[mid] < y: lo = mid + 1
            else: hi = mid
        lo

    evs.sort(by: e => e.x)
    var st = CoverTree.new(uniq)
    var area = 0
    var prev_x = evs[0].x
    for ev in evs:
        area += (ev.x - prev_x) * st.covered
        st.add(y_id(ev.y1), y_id(ev.y2), ev.delta)
        prev_x = ev.x
    area

fn solve() -> Int:
    union_area(q) - union_area(q - 1)

@brute(solve)
fn solve_brute() -> Int
    where n <= 60 and m <= 60:
    var ans = 0
    for r in 1..=n:
        for c in 1..=m:
            let d = min(max(abs(r - zombies[z].r), abs(c - zombies[z].c)) for z in Z)
            if d == q: ans += 1
    ans

print(solve())

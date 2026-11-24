// Tuned: the Li Chao tree written out over arrays allocated once and reused for every lam,
// with 32-bit counts and links; std LiChao regrows ?Photo cells of 48 bytes per lam. The
// outermost ranges come from one in-place sort, without outermost()'s indexed copy.
//
// Source: IOI 2016 Day 2 (Aliens), standard I/O version, https://dmoj.ca/problem/ioi16p6io
// limits: 1000 ms, 256 MB
// Problem: n <= 1e5 points (r, c) in an m x m grid, m <= 1e6. Take at most k <= n photos, each
//   a square with two opposite corners on the main diagonal, covering every point; print the
//   least number of cells photographed at least once.
// Solution: point (r, c) needs the diagonal range [min, max]; drop ranges inside others and the
//   rest sort with both ends increasing, so each photo covers a run of them. Aliens trick: charge
//   lam per photo and binary search the least lam whose optimum uses <= k photos, then answer
//   cost - k * lam. For fixed lam a DP from the right takes a run i..j-1 for (r_{j-1}+1 - l_i)^2
//   minus the overlap with range i-1; the square is a line in l_i, so a Li Chao tree over 0..=m
//   (ties toward fewer photos) makes it O(n log m) per lam, O(n log m log m^2) overall.
// original: corpus/submissions/4730558.cpp
// The square from cut l to cut r costs (r - l)^2, a line in l at r; each line carries its photo
// count, and the LiChao key (val, count) breaks equal costs toward fewer photos, as `aliens` needs.
// Bounds: n, k: 1..=1e5; m: 1..=1e6; spans.r, spans.c: 0..m
let (n, m, k) = read<(1..=1e5, 1..=1e6, 1..=1e5)>()
domain Diag = 0..m
var ranges = read<[n](r: Diag, c: Diag)>().map(p => min(p.r, p.c)..=max(p.r, p.c))
// outermost, sorted in place: by left end, the longer first, keeping a range only when it
// reaches past every one before it
ranges.sort(by: s => (s.l, -Int(s.r)))
var spans: List<Range<Diag>> = []
for s in ranges if spans.is_empty or spans.last!.r < s.r: spans.push(s)

// the photos' lines in one Li Chao tree over 0..=m: cell i holds y = slope * x + base with
// count photos, and its children (0 for none, the root being cell 0); an add makes at most
// one cell, so spans.len + 1 cells always suffice and every lam reuses them
let cells = spans.len + 1
var slope: [cells]Int = 0
var base: [cells]Int = 0
var count: [cells]0..=100_000 = 0
var lower: [cells]0..=100_001 = 0
var upper: [cells]0..=100_001 = 0
var used = 0
// (val, count) order, so equal costs break toward fewer photos, as `aliens` needs
fn below(v: Int, c: Int, w: Int, d: Int) -> Bool = v < w or v == w and c < d
fn add(a: Int, b: Int, c: 0..=100_000):
    var (sa, sb, sc) = (a, b, c)
    if used == 0:
        (slope[0], base[0], count[0], lower[0], upper[0]) = (sa, sb, sc, 0, 0)
        used = 1
        return
    var (i, l, r) = (0, 0, m)
    unchecked:
        while true:
            let mid = l + (r - l) / 2
            let left_wins = below(sa * l + sb, sc, slope[i] * l + base[i], count[i])
            let mid_wins = below(sa * mid + sb, sc, slope[i] * mid + base[i], count[i])
            if mid_wins:
                (sa, slope[i]) = (slope[i], sa)
                (sb, base[i]) = (base[i], sb)
                (sc, count[i]) = (count[i], sc)
            if l == r: return
            let go_left = left_wins != mid_wins
            let kid = if go_left: lower[i] else: upper[i]
            if kid == 0:
                (slope[used], base[used], count[used], lower[used], upper[used]) = (sa, sb, sc, 0, 0)
                if go_left: lower[i] = used else: upper[i] = used
                used += 1
                return
            i = kid
            if go_left: r = mid else: l = mid + 1
fn least(x: Int) -> (val: Int, count: Int):
    var best = (val: slope[0] * x + base[0], count: Int(count[0]))
    var (i, l, r) = (0, 0, m)
    unchecked:
        while true:
            let v = slope[i] * x + base[i]
            if below(v, count[i], best.val, best.count): best = (val: v, count: Int(count[i]))
            let mid = l + (r - l) / 2
            if x <= mid:
                i = lower[i]
                r = mid
            else:
                i = upper[i]
                l = mid + 1
            if i == 0: break
    best
fn cover(fee: Int) -> (val: Int, count: Int):
    used = 0
    var best = (val: 0, count: 0)
    for (prev, s) in spans.with_prev:
        let (lo, x) = (Int(s.l), Int(s.r))
        let shared = prev.map(p => p.clip(s).len) ?? 0
        add(-2 * lo, best.val + lo * lo - shared * shared + fee, best.count + 1)
        let b = least(x)
        best = (val: b.val + x * x, count: b.count)
    best
print(aliens(k, cover, lambdas: 0..=m * m))

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
let (n, m, k) = read<(1..=1e5, 1..=1e6, 1..=1e5)>()
domain Diag = 0..m
let spans = read<[n](r: Diag, c: Diag)>().map(p => min(p.r, p.c)..=max(p.r, p.c)).outermost()

struct Photo(line: Line, count: Int):
    fn at(self, x: Int) -> (val: Int, count: Int) = (val: line.at(x), count:)

fn cover(fee: Int) -> (val: Int, count: Int):
    var photos = LiChao<Photo, (val: Int, count: Int)>(min, over: 0..=m)
    var best = (val: 0, count: 0)
    for (prev, s) in spans.with_prev:
        let (lo, x) = (Int(s.l), Int(s.r))
        let shared = prev.map(p => p.clip(s).len) ?? 0
        photos.add(Photo(Line(-2 * lo, best.val + lo * lo - shared * shared + fee), best.count + 1))
        let b = photos.at(x)!
        best = (val: b.val + x * x, count: b.count)
    best

print(aliens(k, cover, lambdas: 0..=m * m))

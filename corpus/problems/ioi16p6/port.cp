// Source: IOI 2016 P6 (Aliens), https://dmoj.ca/problem/ioi16p6
// Problem: n <= 1e5 points on an m x m grid, m <= 1e6. A photo is a square whose two opposite
//   corners lie on the main diagonal; with at most k photos covering every point, print the least
//   number of cells photographed at least once. Input as the IOI sample grader gives it.
// Solution: point (r, c) needs the diagonal span min..max; sorted by start with contained spans
//   dropped, both ends increase and a photo covers a run. done[i] = min over j of done[j - 1] +
//   (hi_i - lo_j + 1)^2 - (overlap of spans j - 1, j)^2 + fee is a monotone convex hull trick,
//   ties to fewer photos. Binary search the smallest fee with at most k photos and print
//   cost - k * fee (Aliens trick): O(n log m).
// original: corpus/submissions/4730545.cpp
// The square from cut l to cut r costs (r - l)^2, a line in l at r; each line carries its photo
// count so equal costs prefer fewer photos, which `aliens` needs.
// Bounds: n, k: 1..=1e5; m: 1..=1e6
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

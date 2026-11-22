// ioi16p6 Aliens: cover points with at most k diagonal squares of least total area. A point needs
// the diagonal span min(r, c)..=max(r, c); drop spans inside others, and a photo covers a run of
// the rest. Pay `fee` per photo to lift the count limit (`aliens`); each run's cost is a line in
// its first span's left end, so the DP is a monotone hull whose lines carry their photo counts.
let (n, m, k) = read<(1..=100_000, 1..=1_000_000, 1..=100_000)>()
var spans: List<(lo: Int, hi: Int)> = []
for s in read<[n](r: 0..m, c: 0..m)>().map(p => (lo: min(p.r, p.c), hi: max(p.r, p.c))).sorted(by: s => (s.lo, -s.hi)):
    if spans.len == 0 or s.hi > spans.last!.hi:
        spans.push(s)
fn cover(fee: Int) -> (val: Int, count: Int):
    var hull = MonotoneHull<Int>()
    var best = (val: 0, count: 0)
    for (i, s) in spans.items:
        let shared = if i > 0: max(0, spans[i - 1].hi - s.lo + 1) else: 0
        hull.add(Line(-2 * s.lo, best.val + s.lo * s.lo - shared * shared + fee), tag: best.count + 1)
        let x = s.hi + 1
        let b = hull.best_next(x)!
        best = (val: b.val + x * x, count: b.tag)
    best
print(aliens(k, cover, lambdas: 0..=m * m))

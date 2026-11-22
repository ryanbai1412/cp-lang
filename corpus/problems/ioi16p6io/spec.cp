// IOI 2016 "Aliens" (stdin): cover every point's diagonal range with at most k squares
// on the diagonal, minimizing covered cells. Aliens trick over a monotone-hull DP.
let (n, m, k) = read<(1..=1e5, 1..=1e6, 1..=1e5)>()
domain Diag = 0..m
let spans = read<[n](r: Diag, c: Diag)>()
    .map(p => min(p.r, p.c)..=max(p.r, p.c))
    .outermost()

fn cover(fee: Int) -> (val: Int, count: Int):
    var hull = MonotoneHull<Int>()
    var best = (val: 0, count: 0)
    for (prev, s) in spans.with_prev:
        let (lo, x) = (Int(s.l), Int(s.r))
        let shared = prev.map(p => p.clip(s).len) ?? 0
        hull.add(Line(-2 * lo, best.val + lo * lo - shared * shared + fee), tag: best.count + 1)
        let b = hull.best_next(x)!
        best = (val: b.val + x * x, count: b.tag)
    best

print(aliens(k, cover, lambdas: 0..=m * m))

// ioi16p6 Aliens: cover points with at most k diagonal squares of least total area. A point needs
// the diagonal range min(r, c)..=max(r, c); drop ranges inside others, and a photo covers a run of
// the rest. Pay `fee` per photo to lift the count limit (`aliens`); a photo from range j's left
// cut to range i's right cut costs (Int(i.r) - Int(j.l))², a line in Int(j.l) evaluated at
// Int(i.r), so the DP is a monotone hull whose lines carry their photo counts.
let (n, m, k) = read<(1..=100_000, 1..=1_000_000, 1..=100_000)>()
domain Diag = 0..m
let all = read<[n](r: Diag, c: Diag)>().map(p => min(p.r, p.c)..=max(p.r, p.c)).sorted(by: s => (s.l, desc(s.r)))
let reach = all.map(s => s.r).scan(max)
let spans = all.keys.filter(i => all[i].r > reach[i.l]).map(i => all[i]).collect()

fn cover(fee: Int) -> (val: Int, count: Int):
    var hull = MonotoneHull<Int>()
    var best = (val: 0, count: 0)
    for (prev, s) in spans.with_prev:
        let lo = Int(s.l)
        let shared = prev.map(p => p.clip(s).len) ?? 0
        hull.add(Line(-2 * lo, best.val + lo * lo - shared * shared + fee), tag: best.count + 1)
        let x = Int(s.r)
        let b = hull.best_next(x)!
        best = (val: b.val + x * x, count: b.tag)
    best

print(aliens(k, cover, lambdas: 0..=m * m))

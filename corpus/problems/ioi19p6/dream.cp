// IOI 2019 "Sky Walking": buildings at x with heights h, skywalks at height y
// between buildings l and r; shortest walk from the base of s to the base of g.
pub fn min_distance(x: List<Int>, h: List<Int>, l: List<Int>, r: List<Int>, y: List<Int>, s: Int, g: Int) -> Int:
    domain B = [x.len]
    let tall = SparseTable(max, [B] b => h[b])
    let walks = List.zip(l, r, y).map((a, b, z) => (span: B(a)..=B(b), y: z))

    // a walk over s or g is cut at the nearest building on each side tall enough to hold it
    fn cut(w, at: B) -> List<(span: RangeInclusive<B>, y: Int)>:
        if not (w.span.l < at < w.span.r):
            return [w]
        let left = (w.span.l..=at).last_true(b => tall.fold(b..=at) >= w.y)!
        let right = (at..=w.span.r).first_true(b => tall.fold(at..=b) >= w.y)!
        [(span: w.span.l..=left, y: w.y), (span: left..=right, y: w.y), (span: right..=w.span.r, y: w.y)]
            .filter(p => p.span.l < p.span.r).collect()
    let walks = walks.flat_map(w => cut(w, B(s))).flat_map(w => cut(w, B(g))).collect()

    // stops: walk ends, plus where each end's building meets the walk just below
    let below = walks.sorted(by: .y).sweep_below(at: (w, b) => w.span.contains(b) and h[b] >= w.y)
    var stops = Set<(B, Int)>([(B(s), 0), (B(g), 0)])
    for w in walks:
        for b in [w.span.l, w.span.r]:
            stops.add((b, w.y))
            for under in below(b, w.y):
                stops.add((b, under.y))

    domain Stop = stops.keys
    let along = walks.flat_map(w => stops.range((w.span.l, w.y)..=(w.span.r, w.y)).filter(p => p.1 == w.y).pairs())
    let up = stops.group(by: .0).values.flat_map(ps => ps.sorted(by: .1).pairs())
    let g2 = Graph.undirected(chain(along, up).map((p, q) => (u: Stop(p), v: Stop(q), w: abs(x[p.0] - x[q.0]) + abs(p.1 - q.1))))
    dijkstra(g2, from: [(Stop((B(s), 0)), 0)]).dist[Stop((B(g), 0))] ?? -1

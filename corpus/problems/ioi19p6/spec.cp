// IOI 2019 "Sky Walking": buildings at x with heights h, skywalks at height y
// between buildings l and r; shortest walk from the base of s to the base of g.
// Walks over s or g are cut at the nearest building on each side tall enough
// to hold them; then the only useful stops are walk ends and, at each end's
// building, the highest walk passing just below it. Dijkstra over those stops.
@grader
fn min_distance(
    x: List<Int>,
    h: List<Int>,
    l: List<Int>,
    r: List<Int>,
    y: List<Int>,
    s: Int,
    g: Int,
) -> Int:
    domain B = [x.len]
    let tall = SparseTable(max, [B] b => h[Int(b)])
    let (start, goal) = (B(s), B(g))

    fn cut(w: (span: Range<B>, y: Int), at: B) -> List<(span: Range<B>, y: Int)>:
        if not (w.span.first! < at < w.span.last!):
            return [w]
        let left = (w.span.first!..=at).last_true(b => tall.fold(b..=at) >= w.y)!
        let right = (at..=w.span.last!).first_true(b => tall.fold(at..=b) >= w.y)!
        [
            (span: w.span.first!..=left, y: w.y),
            (span: left..=right, y: w.y),
            (span: right..=w.span.last!, y: w.y),
        ]
            .filter(p => p.span.len > 1).collect()

    let raw = [(span: B(l[i])..=B(r[i]), y: y[i]) for i in 0..l.len]
    let walks = raw.flat_map(w => cut(w, start)).flat_map(w => cut(w, goal)).collect()

    var stops: List<(b: B, y: Int)> = [(b: start, y: 0), (b: goal, y: 0)]
    var on: List<List<B>> = [List() for _ in walks]
    let starting = walks.map(w => w.span.first!).buckets()
    let ending = walks.map(w => w.span.last!).buckets()
    var active = SortedBag<(y: Int, walk: Int)>()
    for b in B:
        for i in starting[b]:
            active.add((y: walks[i].y, walk: i))
        for i in starting[b] + ending[b]:
            stops.push((b:, y: walks[i].y))
            on[i].push(b)
            if let Some(under) = active.last_true(a => a.y < walks[i].y):
                stops.push((b:, y: under.y))
                on[under.walk].push(b)
        for i in ending[b]:
            active.remove((y: walks[i].y, walk: i))

    let (domain Stop, at) = compress(stops)
    var links: List<(u: Stop, v: Stop, w: Int)> = []
    for i in walks.keys:
        for (p, q) in on[i].sorted().dedup().adjacent:
            links.push(
                (
                    u: at.at((b: p, y: walks[i].y)),
                    v: at.at((b: q, y: walks[i].y)),
                    w: x[Int(q)] - x[Int(p)],
                )
            )
    for (p, q) in Stop.with_next:
        if let Some(q) = q:
            if at.get(p).b == at.get(q).b:
                links.push((u: p, v: q, w: at.get(q).y - at.get(p).y))

    let net = Graph<Stop>.undirected([links.keys] e => links[e])
    dijkstra(net, from: [(at.at((b: start, y: 0)), 0)]).get(at.at((b: goal, y: 0))) ?? -1

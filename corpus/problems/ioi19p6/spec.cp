// IOI 2019 "Sky Walking": buildings at x with heights h, skywalks at height y
// between buildings l and r; shortest walk from the base of s to the base of g.
// Walks over s or g are cut at the nearest building on each side tall enough
// to hold them; then the only useful stops are walk ends and, at each end's
// building, the highest walk passing just below it. Dijkstra over those stops.
//
// The task is a grader function; this reads the sample grader's input
// (`n m`, n lines `x h`, m lines `l r y`, then `s g`) and prints the
// answer, -1 when g is unreachable.
let (n, m) = read<(1..=1e5, 1..=1e5)>()
domain B = [n]
let towers = read<[B](x: 0..=1e9, h: 1..=1e9)>()
let raw = read<[m](l: B, r: B, y: 1..=1e9)>()
let (start, goal) = read<(B, B)>()
let tall = SparseTable(max, [B] b => towers[b].h)

fn cut(w: (span: Range<B>, y: Int), at: B) -> List<(span: Range<B>, y: Int)>:
    if not (w.span.first! < at < w.span.last!):
        return [w]
    let left = (w.span.first!..=at).last_true(b => tall.fold(b..=at) >= w.y)!
    let right = (at..=w.span.last!).first_true(b => tall.fold(at..=b) >= w.y)!
    [
        (span: w.span.first!..=left, y: w.y),
        (span: left..=right, y: w.y),
        (span: right..=w.span.last!, y: w.y),
    ].filter(p => p.span.len > 1).collect()

let over_start = [p for w in raw for p in cut((span: w.l..=w.r, y: w.y), start)]
let walks = [p for w in over_start for p in cut(w, goal)]

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
    for (p, q) in on[i].deduped().adjacent:
        links.push(
            (
                u: at.at((b: p, y: walks[i].y)),
                v: at.at((b: q, y: walks[i].y)),
                w: towers[q].x - towers[p].x,
            )
        )
for (p, q) in Stop.collect().adjacent if at.get(p).b == at.get(q).b:
    links.push((u: p, v: q, w: at.get(q).y - at.get(p).y))

let edges = [links.keys] e => links[e]
let net = Graph.undirected(edges)
print(dijkstra(net, from: [(at.at((b: start, y: 0)), 0)]).get(at.at((b: goal, y: 0))) ?? -1)

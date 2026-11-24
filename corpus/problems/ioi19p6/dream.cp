// IOI 2019 "Sky Walking": buildings at x with heights h, skywalks at height y
// between buildings l and r; shortest walk from the base of s to the base of g.
//
// A walk over s or g is cut at the nearest building on each side tall enough
// to hold it; then the only useful stops are walk ends and, at each end's
// building, the highest walk passing just below it. Dijkstra over those stops.
// Reads the sample grader's input (`n m`, n lines `x h`, m lines `l r y`, then
// `s g`) and prints the answer, -1 when g is unreachable.
let (n, m) = read<(1..=1e5, 1..=1e5)>()
domain B = [n]
let towers = read<[B](x: 0..=1e9, h: 1..=1e9)>()
let raw = read<[m](l: B, r: B, y: 1..=1e9)>()
let (s, g) = read<(B, B)>()
let tall = SparseTable(max, [B] b => towers[b].h)
type Walk = (span: Range<B>, y: Int)

fn cut(w: Walk, at: B) -> List<Walk>:
    let (l, r) = (w.span.first!, w.span.last!)
    if not (l < at < r):
        return [w]
    let left = (l..=at).last_true(b => tall.fold(b..=at) >= w.y)!
    let right = (at..=r).first_true(b => tall.fold(at..=b) >= w.y)!
    [(span: l..=left, y: w.y), (span: left..=right, y: w.y), (span: right..=r, y: w.y)]
        .filter(p => p.span.len > 1).collect()

let walks = [q for w in raw for p in cut((span: w.l..=w.r, y: w.y), s) for q in cut(p, g)]

// sweep the buildings left to right with the walks overhead, ordered by height
var stops: List<(b: B, y: Int)> = [(b: s, y: 0), (b: g, y: 0)]
var on: List<List<B>> = [List() for _ in walks]
let starting = walks.map(w => w.span.first!).buckets()
let ending = walks.map(w => w.span.last!).buckets()
var overhead = SortedBag<(y: Int, walk: Int)>()
for b in B:
    for i in starting[b]: overhead.add((y: walks[i].y, walk: i))
    for i in starting[b] + ending[b]:
        stops.push((b:, y: walks[i].y))
        on[i].push(b)
        if let Some(under) = overhead.last_true(a => a.y < walks[i].y):
            stops.push((b:, y: under.y))
            on[under.walk].push(b)
    for i in ending[b]: overhead.remove((y: walks[i].y, walk: i))

let (domain Stop, at) = compress(stops)
let stop = (b: B, y: Int) => at.index_of((b:, y:))
var links: List<(u: Stop, v: Stop, w: Int)> = []
for (i, w) in walks.items:
    for (p, q) in on[i].deduped().adjacent:
        links.push((u: stop(p, w.y), v: stop(q, w.y), w: towers[q].x - towers[p].x))
for (p, q) in Stop.collect().adjacent if at.get(p).b == at.get(q).b:
    links.push((u: p, v: q, w: at.get(q).y - at.get(p).y))

let edges = [links.keys] e => links[e]
let net = Graph.undirected(edges)
print(dijkstra(net, from: [(stop(s, 0), 0)]).get(stop(g, 0)) ?? -1)

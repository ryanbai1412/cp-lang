// IOI 2012 Ideal City: n blocks on a grid form a city that is connected and
// has no holes. The distance between two blocks is the fewest steps between
// them through blocks. Sum it over all pairs, modulo 10^9.
// original: proto/ports/scout_batch_17/harness/ioi12p4_main.cpp (wraps corpus/submissions/4951048.cpp)
let n: 1..=1e5 = read()
domain Block = 0..n
type At = (x: 0..=2e9, y: 0..=2e9)

// "Connected, no holes" is not a type. This builds one kind of such city:
// each street x holds one unbroken run of blocks that touches the run before.
fn city() -> [Block]At:
    var at: [Block]At = (0, 0)
    var placed = 0
    var x = 0
    var lo = 1e6
    var hi = 1e6
    while placed < n:
        let tall = rand(1, min(n - placed, 5))
        if placed > 0: lo = rand(lo - tall + 1, hi)
        hi = lo + tall - 1
        for y in lo..=hi:
            at[Block(placed)] = (x, y)
            placed += 1
        x += 1
    at

let at: [Block]At = read(gen: city)

// Steps east or west, summed over all pairs. Cut each street (one x) into
// runs of touching blocks. Without holes the runs form a tree, and a shortest
// way between two blocks crosses exactly the tree edges between their runs.
// So an edge with w blocks on one side is crossed by w * (n - w) pairs.
fn crossings(blocks: List<(Int, Int)>) -> Int:
    var run_of: Map<(Int, Int), Int> = []  // a HashMap keyed by a pair builds, then fails in g++
    var size: List<Int> = []
    var last = (-1, -1)
    for (x, y) in blocks.sorted():
        if last != (x, y - 1): size.push(0)
        size.last! += 1
        run_of[(x, y)] = size.len - 1
        last = (x, y)

    domain Run = 0..size.len
    var touch: [Run]List<Run> = []
    for (x, y) in blocks:
        if let west = run_of.get((x - 1, y)):
            // one edge per pair of runs: skip it when the blocks just below already made it
            if not ((x, y - 1) in run_of and (x - 1, y - 1) in run_of):
                let here = run_of.get((x, y)) ?? 0
                touch[Run(west)].push(Run(here))
                touch[Run(here)].push(Run(west))
    let tree = Rooted(touch, Run.first!)
    var under: [Run]Int = [Run] r => size[r.label]
    var total = 0
    for r in tree.at.rev:
        if let p = tree.parent(r):
            total += under[r] * (n - under[r])
            under[p] += under[r]
    total

var across: List<(Int, Int)> = []
var along: List<(Int, Int)> = []
for (x, y) in at:
    across.push((x, y))
    along.push((y, x))
print((crossings(across) + crossings(along)) % 1e9)

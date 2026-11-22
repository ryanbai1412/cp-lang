// Educational DP Z, Frog 3: stones 1..n at increasing heights. A jump from
// stone j to a later stone i costs (h[i] - h[j])^2 + c. Cheapest way to reach
// stone n from stone 1.
// original: corpus/submissions/4133674.cpp
let n: 2..=2e5 = read()
let c: 1..=1e12 = read()
domain Stone = 1..=n

fn climbing() -> [Stone]1..=1e6:
    var up: [Stone]1..=1e6 = 1
    var top = 0
    for s in Stone:
        top += rand(1, 5)
        up[s] = top
    up

let h: [Stone]1..=1e6 = read(gen: climbing)

// cost[i] = c + h[i]^2 + min over j < i of (cost[j] + h[j]^2) - 2 h[j] h[i]:
// the lowest of a set of lines, asked at x = h[i]. A Li Chao tree over the
// stones (x = h[s]) keeps, per node, the line that wins at the node's middle
// stone, so two lines are only ever compared at a stone: no intersection
// points, no products that outgrow Int.
type Line = (slope: Int, at0: Int)
fn value(f: Line, s: Stone) -> Int = f.slope * h[s] + f.at0

domain Node = 1..4 * n
var winner: [Node]?Line = none

fn add(f: Line, node: Node, lo: Stone, hi: Stone):
    let mid = Stone((lo.label + hi.label) / 2)
    let g = winner[node] ?? f
    let (keep, sink) = if value(f, mid) < value(g, mid): (f, g) else: (g, f)
    winner[node] = keep
    if lo == hi: return
    if value(sink, lo) < value(keep, lo): add(sink, Node(2 * node.label), lo, mid)
    elif value(sink, hi) < value(keep, hi): add(sink, Node(2 * node.label + 1), Stone(mid.label + 1), hi)

fn lowest(s: Stone) -> Int:
    var node = Node.first
    var lo = Stone.first
    var hi = Stone.last
    var best: ?Int = none
    while true:
        if let g = winner[node]: best min= value(g, s)
        if lo == hi: break
        let mid = Stone((lo.label + hi.label) / 2)
        if s <= mid:
            node = Node(2 * node.label)
            hi = mid
        else:
            node = Node(2 * node.label + 1)
            lo = Stone(mid.label + 1)
    best ?? 0                               // never empty: stone 1 is added before any query

var cost: [Stone]Int = 0
for s in Stone:
    if s != Stone.first: cost[s] = lowest(s) + h[s] * h[s] + c
    add((-2 * h[s], cost[s] + h[s] * h[s]), Node.first, Stone.first, Stone.last)
print(cost[Stone.last])

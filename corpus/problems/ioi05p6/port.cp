// Source: IOI 2005 (Rivers), https://dmoj.ca/problem/ioi05p6
// limits: 600 ms, 32 MB
// Problem: n <= 100 villages on a river tree draining to Bytetown (0), which has a sawmill;
//   village i cuts w_i trees and flows to v_i, d_i km away. Build k <= 50 more mills in
//   villages; trees float to the first mill downriver at 1 cent per tree-km. Print the least cost.
// Solution: the cost of a village depends only on its nearest milled ancestor, so cost(v, mill)
//   is a memoized Row over j = 0..=k mills in v's subtree: either v ships to `mill`, its children
//   seeing `mill`, or v builds one and they see v. spread() min-plus merges the children's rows,
//   O(k^2) per child, so O(n^2 k^2) overall; the answer is Bytetown's spread with k mills.
// cp: 0.1 8ef64b9+dirty
// Bounds: n: 2..=100; k: 1..=50
let n: 2..=100 = read()
let k: 1..=50 = read()
domain Village = 1..=n
domain Site = 0..=n  // where logs can go: Bytetown (label 0) or a village
fn site(v: Village) -> Site = Site.at(v.label)!
fn village(s: Site) -> ?Village = if s.label == 0: None else: Village.at(s.label)
type Info = (trees: 0..=1e4, down: Site, len: 1..=1e4)

fn drains(rows: [Village]Info) -> Bool:
    for v in Village:
        var s = site(v)
        var steps = 0
        while let u = village(s):
            s = rows[u].down
            steps += 1
            if steps > n:
                return false
    true

fn random_river() -> [Village]Info:
    [Village] v => (
        trees: rand(0, 10000),
        down: Site.at(rand(0, v.label - 1))!,
        len: rand(1, 10000),
    )

let rows: [Village]Info = read(gen: random_river)
assert drains(rows)

var kids: [Site]List<Village> = []
for v in Village:
    kids[rows[v].down].push(v)

@memo
fn depth(s: Site) -> Int:
    if let v = village(s):
        return depth(rows[v].down) + rows[v].len
    0

domain Mills = 0..=k
type Row = [Mills]Int  // row[j]: best cost with at most j new mills

// min-plus combination of the children's rows, their logs going to `mill`
fn spread(cs: List<Village>, mill: Site) -> Row:
    var best: Row = 0
    for c in cs:
        let sub = cost(c, mill)
        best = [Mills] t => [best[a] + sub[Mills(t - a)] for a in ..=t].min()!
    best

// v's subtree; v's logs reach `mill` unless v builds its own
@memo
fn cost(v: Village, mill: Site) -> Row:
    let haul = rows[v].trees * (depth(site(v)) - depth(mill))
    let open = spread(kids[site(v)], mill)
    let milled = spread(kids[site(v)], site(v))
    [Mills] j => if j == Mills.first: haul + open[j] else: min(haul + open[j], milled[j.prev!])

print(spread(kids[Site.first!], Site.first!)[Mills.last!])

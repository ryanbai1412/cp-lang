// cp: 0.1 8ef64b9+dirty
// IOI 2005 Rivers. Villages drain down a river tree into Bytetown, which
// already has a sawmill. Build at most k more mills in villages to minimise
// the sum of trees[v] * (distance from v down to the first mill it reaches).
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

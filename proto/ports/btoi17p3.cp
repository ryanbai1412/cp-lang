// BOI 2017 Railway: each minister wants the tracks that connect a set of
// cities. Report the tracks wanted by at least k ministers.
let (n, m, k): (2..=1e5, 1..=5e4, 1..=5e4) = read()
domain City = 1..=n
domain Track = 1..n
let tracks: [Track](u: City, v: City) = read()
let tree = Rooted(Graph<City, Track>.undirected(tracks), City.first)

// Walking a plan's cities in preorder and back to the start crosses every
// wanted track exactly twice. Mark both ends of each leg and cancel at their
// meeting point: the marks below a track then add up to twice its ministers.
var marks: [City]Int = 0
for _ in 0..m:
    let s: 2..=1e5 = read()
    let plan: [0..s]City = read()
    let tour = sorted(plan, by: c => tree.pos(c))
    for i in 0..s:
        let (a, b) = (tour[i], tour[(i + 1) % s])
        marks[a] += 1
        marks[b] += 1
        marks[tree.lca(a, b)] -= 2
for c in tree.postorder:                    // children before parents
    if let p = tree.parent(c): marks[p] += marks[c]

fn lower(t: Track) -> City = if tree.parent(tracks[t].u) == tracks[t].v: tracks[t].u else: tracks[t].v
let wanted = [t for t in Track if marks[lower(t)] >= 2 * k]
print(wanted.len)
print(wanted)

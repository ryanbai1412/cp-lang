// cp: 0.1 0abd108+dirty
// BOI 2017 Railway: each minister wants the tracks that connect a set of
// cities. Report the tracks wanted by at least k ministers.
let n: 2..=1e5 = read()
let m: 1..=5e4 = read()
let k: 1..=m = read()
domain City = 1..=n
domain Track = 1..n

fn random_tree() -> [Track](u: City, v: City):
    [Track] t => (City(rand(1, t.label)), City(t.label + 1))

let tracks: [Track](u: City, v: City) = read(gen: random_tree)
let plans: [m]List<City> = read()

var adj: [City]List<City> = []
for (u, v) in tracks:
    adj[u].push(v)
    adj[v].push(u)
let tree = Rooted(adj, City.first)

// Walking a plan's cities in preorder and back to the start crosses every
// wanted track exactly twice. Mark both ends of each leg and cancel at their
// meeting point: the marks below a track then add up to twice its ministers.
var marks: [City]Int = 0
for plan in plans:
    let tour = sorted(plan, by: c => tree.pos(c))
    for i in 0..tour.len:
        let (a, b) = (tour[i], tour[(i + 1) % tour.len])
        marks[a] += 1
        marks[b] += 1
        marks[tree.lca(a, b)] -= 2
for c in tree.preorder.rev:                 // children before parents
    if let p = tree.parent(c): marks[p] += marks[c]

fn lower(t: Track) -> City:
    if tree.parent(tracks[t].u) == tracks[t].v: tracks[t].u else: tracks[t].v
let wanted = [t for t in Track if marks[lower(t)] >= 2 * k]
print(wanted.len)
print(wanted)

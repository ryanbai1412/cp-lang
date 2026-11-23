// COCI 2021/22 #4 Parkovi, as it builds today: the parks of the best radius are
// worked out a second time (R2), and the road lengths are stored beside the tree (B14).
// original: corpus/submissions/5120081.cpp
let n: 1..=2e5 = read()
let k: 1..=n = read()
domain Town = 1..=n
domain Road = 1..n

fn random_tree() -> [Road](u: Town, v: Town, w: 1..=1e9):
    [Road] t => (Town(rand(1, t.label)), Town(t.label + 1), rand(1, 20))

let roads: [Road](u: Town, v: Town, w: 1..=1e9) = read(gen: random_tree)

var adj: [Town]List<Town> = []
for (u, v, w) in roads:
    adj[u].push(v)
    adj[v].push(u)
let tree = Rooted(adj, Town.first)
var up: [Town]Int = 0  // length of the road to the parent
for (u, v, w) in roads:
    up[if tree.parent(u) == v: u else: v] = w

// The fewest parks that leave every town within `reach` of one: going up
// the tree, a park is put down only when the farthest unserved town below
// could not reach anything higher.
fn parks(reach: Int) -> List<Town>:
    var built: List<Town> = []
    var far: [Town]?Int = 0      // farthest town at or below v that no park serves yet
    var near: [Town]?Int = none  // nearest park at or below v
    for v in tree.at.rev:        // children before parents
        if let f = far[v]:
            if let d = near[v]:
                if f + d <= reach: far[v] = none
        if let p = tree.parent(v):
            if let f = far[v]:
                if f + up[v] > reach:
                    built.push(v)
                    near[v] = 0
                    far[v] = none
            if let f = far[v]: far[p] max= f + up[v]
            if let d = near[v]: near[p] min= d + up[v]
        elif far[v] != none:
            built.push(v)
    built

let reach = bsearch_first(0..=2e14, x => parks(x).len <= k)!
print(reach)

var chosen = parks(reach)
var park: [Town]Bool = false
for t in chosen: park[t] = true
for t in Town:
    if chosen.len < k and not park[t]: chosen.push(t)
print(chosen)

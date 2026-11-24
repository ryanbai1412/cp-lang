// Source: COCI 2021/22 Round 4 (Parkovi), https://dmoj.ca/problem/coci21c4p4
// limits: 3000 ms, 512 MB
// Problem: a tree on n <= 2e5 towns, road lengths <= 1e9. Build exactly k parks in distinct
//   towns minimising the largest distance from a town to its nearest park; print that distance,
//   then the k towns.
// Solution: binary search the radius over 0..=2e14 with a greedy check: going up the tree,
//   keep the farthest unserved town and the nearest park in each subtree, and build a park at
//   v only when the farthest unserved town below could not reach v's parent (always at the
//   root if one is left). That gives the fewest parks for a radius, O(n) per check, O(n log)
//   in all; pad the answer with any other towns up to k.
// original: corpus/submissions/5120081.cpp
// Bounds: n: 1..=2e5; k: 1..=n
let n: 1..=2e5 = read()
let k: 1..=n = read()
domain Town = 1..=n
domain Road = 1..n

fn random_tree() -> [Road](u: Town, v: Town, w: 1..=1e9):
    [Road] t => (Town(rand(1, t.label)), Town(t.label + 1), rand(1, 20))

let roads: [Road](u: Town, v: Town, w: 1..=1e9) = read(gen: random_tree)
let tree = Rooted(Graph.undirected(roads), Town.first!)

// The fewest parks that leave every town within `reach` of one: going up
// the tree, a park is put down only when the farthest unserved town below
// could not reach anything higher.
fn parks(reach: Int) -> List<Town>:
    var built: List<Town> = []
    var far: [Town]?Int = 0      // farthest town at or below v that no park serves yet
    var near: [Town]?Int = None  // nearest park at or below v
    for v in tree.order.rev:     // children before parents
        if let f = far[v]:
            if let d = near[v]:
                if f + d <= reach: far[v] = None
        if let up = tree.up_arc(v):
            let p = tree.parent(v)!
            if let f = far[v]:
                if f + up.w > reach:
                    built.push(v)
                    near[v] = 0
                    far[v] = None
            if let f = far[v]: far[p] max= f + up.w
            if let d = near[v]: near[p] min= d + up.w
        elif far[v] != None:
            built.push(v)
    built

let reach = (0..=2e14).first_true(x => parks(x).len <= k)!
print(reach)

var chosen = parks(reach)
var park: [Town]Bool = false
for t in chosen: park[t] = true
for t in Town:
    if chosen.len < k and not park[t]: chosen.push(t)
print(chosen)

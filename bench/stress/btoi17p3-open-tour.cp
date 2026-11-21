// BOI 2017 Railway (proto/ports/btoi17p3.cp), with a planted bug: the walk over a plan's cities
// in preorder never returns from the last city to the first, so the tracks up from the last
// city go unmarked. Each plan is read in a loop pass as its size, then that many cities.
use std.graph.{Tree}

let (n, m, k): (2..=1e5, 1..=5e4, 1..=5e4) = read()
domain City = 1..=n
let tracks: Tree<City> = read()
let tree = Rooted(Graph.undirected(tracks), City.first)
var plans: List<List<City>> = []
for _ in 0..m:
    let s: 2..=1e5 = read()
    let plan: [0..s]City = read()
    plans.push([c for c in plan])

// A track is named by its lower city.
fn lower(u: City, v: City) -> City = if tree.parent(u) == v: u else: v

fn wanted(k: Int, plans: List<List<City>>) -> List<Int>:
    var marks: [City]Int = 0
    for plan in plans:
        let tour = sorted(plan, by: tree.pos)
        for i in 0..tour.len - 1:
            marks[tour[i]] += 1
            marks[tree.lca(tour[i], tour[i + 1])] -= 1
    for c in tree.postorder:
        if let p = tree.parent(c): marks[p] += marks[c]
    [i for i in 0..tracks.len if marks[lower(tracks[i].u, tracks[i].v)] >= k]

@brute(wanted)
fn wanted_slow(k: Int, plans: List<List<City>>) -> List<Int>:
    var count: [City]Int = 0
    for plan in plans:
        var used: [City]Bool = false
        for c in plan:
            let top = tree.lca(c, plan[0])
            for start in [c, plan[0]]:
                var x = start
                while x != top:
                    used[x] = true
                    x = tree.parent(x)!
        for c in City:
            if used[c]: count[c] += 1
    [i for i in 0..tracks.len if count[lower(tracks[i].u, tracks[i].v)] >= k]

let found = wanted(k, plans)
print(found.len)
print([i + 1 for i in found])

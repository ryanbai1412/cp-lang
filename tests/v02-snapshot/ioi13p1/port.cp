// Source: IOI 2013 Day 1 (Dreaming), https://dmoj.ca/problem/ioi13p1
// Problem: a forest of N <= 1e5 billabongs and M <= N - 1 trails of length <= 1e4. Add new
//   trails of length L <= 1e4 until it is one tree, keeping the longest travel time between
//   two billabongs as small as possible, and print that time.
// Solution: in each tree, the farthest node a from anywhere and the farthest b from a end
//   every node's longest walk, so three walks give each node's eccentricity, the diameter
//   and the radius (the smallest eccentricity). Hang every centre off the centre of largest
//   radius: the answer is the max of all diameters, r1 + r2 + L and r2 + r3 + 2L for the
//   radii sorted down: O(N + T log T) for T trees.
// original: proto/ports/scout_batch_17/harness/ioi13p1_main.cpp (wraps corpus/submissions/4687433.cpp)
let n: 1..=1e5 = read()
let m: 0..n = read()
let l: 1..=1e4 = read()
domain V = 0..n
domain E = 1..=m
type Trail = (a: V, b: V, t: 1..=1e4)

fn forest() -> [E]Trail:  // the last m billabongs each hang off an earlier one
    [E] e => (V(rand(0, e.label + n - 2 - m)), V(e.label + n - 1 - m), rand(1, 1e4))

let trails: [E]Trail = read(gen: forest)

var adj: [V]List<(V, Int)> = []
for (a, b, t) in trails:
    adj[a].push((b, t))
    adj[b].push((a, t))

type Walk = (to: V, dist: Int)

// Every billabong in the tree of `from` with its distance. A tree has one
// way to each, so remembering where a walk came from is enough to not turn back.
fn spread(from: V) -> List<Walk>:
    var walks: List<Walk> = []
    var todo: List<(at: V, came_from: V, dist: Int)> = [(from, from, 0)]
    while let (u, back, d) = todo.pop():
        walks.push((u, d))
        for (v, t) in adj[u]:
            if v != back: todo.push((v, u, d + t))
    walks

fn farthest(walks: List<Walk>) -> V:  // WISH: max(walks, by: .dist).to
    var far = walks[0]
    for w in walks:
        if w.dist > far.dist: far = w
    far.to

// In each tree: a is farthest from anywhere, b is farthest from a, and every
// billabong's longest walk ends at a or b. The tree's best meeting point is
// the billabong whose longest walk is shortest (the radius).
var reach: [V]Int = 0  // longest walk from each billabong
var seen: [V]Bool = false
var longest = 0
var radii: List<Int> = []
for root in V:
    if not seen[root]:
        let tree = spread(root)
        let from_a = spread(farthest(tree))
        let from_b = spread(farthest(from_a))
        for (v, d) in from_a: reach[v] = d
        for (v, d) in from_b: reach[v] max= d
        for (v, _) in tree: seen[v] = true
        longest max= [d for (_, d) in from_b].max() ?? 0
        radii.push([reach[v] for (v, _) in tree].min() ?? 0)

// Hang every other tree's centre off the centre of the tree with the largest
// radius. The longest new walk joins the two largest radii, or the second and
// third through two new trails.
let r = radii.sorted(by: x => -x)
if r.len >= 2: longest max= r[0] + r[1] + l
if r.len >= 3: longest max= r[1] + r[2] + 2 * l
print(longest)

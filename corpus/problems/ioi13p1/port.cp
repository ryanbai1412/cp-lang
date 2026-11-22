// IOI 2013 Dreaming: n billabongs and m trails form a forest. Add trails of
// length l until it is one tree, so that the longest walk between two
// billabongs is as short as possible. Print that length.
// original: proto/ports/scout_batch_17/harness/ioi13p1_main.cpp (wraps corpus/submissions/4687433.cpp)
let n: 1..=1e5 = read()
let m: 0..n = read()
let l: 1..=1e4 = read()
domain V = 0..n
domain E = 1..=m
type Trail = (a: V, b: V, t: 1..=1e4)

fn forest() -> [E]Trail:                    // the last m billabongs each hang off an earlier one
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

fn farthest(walks: List<Walk>) -> V:        // WISH: max(walks, by: .dist).to
    var far = walks[0]
    for w in walks:
        if w.dist > far.dist: far = w
    far.to

// In each tree: a is farthest from anywhere, b is farthest from a, and every
// billabong's longest walk ends at a or b. The tree's best meeting point is
// the billabong whose longest walk is shortest (the radius).
var reach: [V]Int = 0                       // longest walk from each billabong
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
        longest max= max(d for (_, d) in from_b) ?? 0
        radii.push(min(reach[v] for (v, _) in tree) ?? 0)

// Hang every other tree's centre off the centre of the tree with the largest
// radius. The longest new walk joins the two largest radii, or the second and
// third through two new trails.
let r = radii.sorted(by: x => -x)
if r.len >= 2: longest max= r[0] + r[1] + l
if r.len >= 3: longest max= r[1] + r[2] + 2 * l
print(longest)

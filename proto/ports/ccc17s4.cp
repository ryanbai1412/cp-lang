// CCC 2017 S4 Minimum Cost Flow: the first n - 1 pipes are the current
// plan. Find a cheapest spanning tree that keeps as many old pipes as
// possible; the enhancer (strength d) makes one pipe free, which saves a day
// when an unused old pipe of cost <= d can replace the most expensive new one.
let (n, m, d): (1..=1e5, 1..=2e5, 0..=1e9) = read()
domain V = 1..=n
domain Pipe = 0..m
let pipes: [Pipe](u: V, v: V, cost: 1..=1e9) = read()

fn is_new(e: Pipe) -> Bool = Int(e) >= n - 1
let order = sorted([e for e in Pipe], by: e => (pipes[e].cost, is_new(e)))

var tree = Dsu<V>()
var used: [Pipe]Bool = false
var dearest = 0
for e in order if tree.unite(pipes[e].u, pipes[e].v):
    used[e] = true
    dearest = pipes[e].cost

// the tree without its most expensive new pipes: can a free old pipe rejoin it?
var rest = Dsu<V>()
for e in order if used[e] and (not is_new(e) or pipes[e].cost < dearest): rest.unite(pipes[e].u, pipes[e].v)
let saved = dearest <= d and any(not used[e] and not is_new(e) and pipes[e].cost <= d and not rest.same(pipes[e].u, pipes[e].v) for e in Pipe)

print(count(used[e] and is_new(e) for e in Pipe) - Int(saved))

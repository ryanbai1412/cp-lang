// BOI 2017 Cat in a Tree: mark as many nodes as possible so that every two
// marked nodes are at least d edges apart. Node 0 is the root and every
// parent has a smaller label than its child.
let n: 1..=2e5 = read()
let d: 1..=2e5 = read()
domain V = 0..n
domain Kid = 1..n

fn random_parents() -> [Kid]V:
    [Kid] k => V(rand(0, k.label - 1))

let parent: [Kid]V = read(gen: random_parents)

// marks chosen inside a subtree, and the depth of the shallowest one
type Pick = (count: Int, near: Int)

// Two picks that hang off the same node: keep both if their shallowest marks
// are far enough apart, otherwise give up the shallower mark.
fn join(a: Pick, b: Pick) -> Pick:
    if a.near + b.near >= d: (a.count + b.count, min(a.near, b.near))
    else: (a.count + b.count - 1, max(a.near, b.near))

var pick: [V]Pick = (1, 0)  // on its own, a node marks itself
for k in Kid.rev:           // children are finished before parents
    let below = pick[V(k.label)]
    pick[parent[k]] = join(pick[parent[k]], (below.count, below.near + 1))
print(pick[V.first!].count)

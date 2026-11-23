// original: corpus/submissions/7730380.cpp
// COCI 2025/26 contest 3 problem 4: hang a tree from a vertex r and pick k
// vertices other than r, none of them above another. For every k below n,
// count the picks, summed over every choice of r.
const P = 1000000007
type Z = Mod<P>
let n: 1..=5000 = read()
domain V = 1..=n
domain E = 1..n

fn random_tree() -> [E](u: V, v: V):
    [E] e => (V(rand(1, e.label)), V(e.label + 1))

let edges: [E](u: V, v: V) = read(gen: random_tree)
let tree = Rooted(edges, V.first!)

// A count for every k is a polynomial: the coefficient of x^k is the count.
// Picking independently in two parts multiplies their polynomials.
fn times(a: List<Z>, b: List<Z>) -> List<Z>:
    var c = [Z(0) for _ in 0..a.len + b.len - 1]
    for i in 0..a.len:
        for j in 0..b.len: c[i + j] += a[i] * b[j]
    c

// Take the part b back out of a product a. b starts with 1 (pick nothing).
fn without(a: List<Z>, b: List<Z>) -> List<Z>:
    var q = [a[i] for i in 0..a.len - b.len + 1]
    for i in 0..q.len:
        for j in 1..=min(i, b.len - 1): q[i] -= b[j] * q[i - j]
    q

// A part hanging off an edge: pick inside it, or pick its top vertex alone.
fn hung(inside: List<Z>) -> List<Z>:
    var f = inside
    if f.len == 1: f.push(Z(0))
    f[1] += 1
    f

// down[v]: picks in v's subtree, hanging from v's parent
var down: [V]List<Z> = [V] _ => [Z(1)]
for v in tree.at.rev:
    var inside = [Z(1)]
    for c in tree.children(v): inside = times(inside, down[c])
    down[v] = hung(inside)

// around[v]: picks with the tree hung from v, one factor for every
// neighbour. The parent's factor is everything around the parent except v.
var around: [V]List<Z> = [V] _ => [Z(1)]
var total: [n]Z = 0
for v in tree.at:
    for c in tree.children(v): around[v] = times(around[v], down[c])
    if let p = tree.parent(v): around[v] = times(around[v], hung(without(around[p], down[v])))
    for k in 0..around[v].len: total[k] += around[v][k]
print(total)

// Codeforces 2101 F: Shoo Shatters the Sunshine.
//
// Colour each vertex red, blue or not at all; a colouring is worth the
// largest red-blue distance, and we sum that over all 3^n colourings.
// Subdivide every edge with a midpoint node so that every set of vertices has
// a centre node: the coloured vertices lie within some radius d of a centre c
// and the outermost layer, d from c, is reached in at least two branches of c.
// Each colouring has exactly one such (c, d). With inner layers coloured
// freely and outer ones blank:
// - if the outermost layer holds both colours in different branches, the
//   value is 2d;
// - if it holds one colour (red, say; double for blue) in two or more
//   branches, the value is d + d', where d' < d is the layer of the
//   outermost vertex of the other colour, for some red is d away from it
//   through c.
// Counting layer d in two or more branches is the whole layer minus each
// branch alone. Distances in the subdivided tree are doubled, hence the
// final halving.
type Z = Mod<998_244_353>

@!multitest
let n: 1..=3000 = read()
domain Node = 1..=2 * n - 1
fn is_vertex(v: Node) -> Bool = v.ord < n

var adj: [Node]List<Node> = []
for e in 1..n:
    let (u, v): (Node, Node) = read()
    let mid = Node(n + e)
    adj[u] += [mid]
    adj[v] += [mid]
    adj[mid] += [u, v]

let pow2 = [Z(2) ** k for k in 0..=n]
let pow3 = [Z(3) ** k for k in 0..=n]
fn some_red(c: Int) -> Z = pow2[c] - 1
fn red_and_blue(c: Int) -> Z = pow3[c] - 2 * pow2[c] + 1

// The number of vertices at each depth below `start`, walking away from `parent`.
fn layer_counts(start: Node, parent: Node) -> List<Int>:
    var counts: List<Int> = []
    var layer = [(start, parent)]
    while not layer.is_empty:
        counts.push(layer.count((v, _) => is_vertex(v)))
        layer = [(w, v) for (v, p) in layer for w in adj[v] if w != p]
    counts

var ans: Z = 0
for centre in Node:
    let total = layer_counts(centre, centre)
    let branches = [[0] + layer_counts(c, centre) for c in adj[centre]]
    fn layer(f: Int -> Z, d: Int) -> Z = f(total[d]) - sum(f(b[d]) for b in branches if d < b.len)

    // Over the layers inside d: all colourings, those whose outermost blue
    // vertex exists (layers past it red or blank), and their sum of its depth.
    var (inside, blue_inside, blue_depths): (Z, Z, Z) = (1, 0, 0)
    for (d, c) in total.items:
        ans += inside * layer(red_and_blue, d) * (2 * d) + (blue_depths + blue_inside * d) * layer(some_red, d) * 2
        let outermost_blue = inside * (pow3[c] - pow2[c])
        (blue_inside, blue_depths) = (
            blue_inside * pow2[c] + outermost_blue,
            blue_depths * pow2[c] + outermost_blue * d,
        )
        inside *= pow3[c]
print(ans / 2)

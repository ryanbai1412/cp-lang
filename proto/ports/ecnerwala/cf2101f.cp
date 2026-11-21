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
use lib.math.powers.{Powers}

type Z = Mod<998_244_353>

@!multitest
let n: Int = read()
domain Node = 1..=2 * n - 1
fn is_vertex(v: Node) -> Bool = v.to_idx < n

var adj: [Node]List<Node> = []
for e in 1..n:
    let (u, v): (Node, Node) = read()
    let mid = Node(n + e)
    adj[u] += [mid]
    adj[v] += [mid]
    adj[mid] += [u, v]

let pow2 = Powers<Z.MOD>(2, n)
let pow3 = Powers<Z.MOD>(3, n)
fn some_red(c: Int) -> Z = pow2[c] - 1
fn red_and_blue(c: Int) -> Z = pow3[c] - pow2[c] * 2 + 1

var ans: Z = 0
for centre in Node:
    var total: [0..Node.K]Int = 0
    if is_vertex(centre): total[0] = 1
    var red: [0..Node.K]Z = 0
    var both: [0..Node.K]Z = 0
    for c in adj[centre]:
        var branch: [0..Node.K]Int = 0
        fn count(v: Node, from: Node, depth: Int):
            if is_vertex(v): branch[depth] += 1
            for w in adj[v] if w != from: count(w, v, depth + 1)
        count(c, centre, 1)
        for d in 0..Node.K:
            total[d] += branch[d]
            red[d] -= some_red(branch[d])
            both[d] -= red_and_blue(branch[d])
    for d in 0..Node.K:
        red[d] += some_red(total[d])
        both[d] += red_and_blue(total[d])

    // Over the layers inside d: all colourings, those whose outermost blue
    // vertex exists (layers past it red or blank), and their sum of its depth.
    var inside: Z = 1
    var blue_inside: Z = 0
    var blue_depths: Z = 0
    for d in 0..Node.K:
        let c = total[d]
        ans += inside * both[d] * (2 * d)
        ans += (blue_depths + blue_inside * d) * red[d] * 2
        let outermost_blue = inside * (pow3[c] - pow2[c])
        blue_inside = blue_inside * pow2[c] + outermost_blue
        blue_depths = blue_depths * pow2[c] + outermost_blue * d
        inside *= pow3[c]
print(ans / 2)

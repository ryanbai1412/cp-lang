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
use std.math.{Powers}

type Z = Mod<998_244_353>

@!multitest
let n: Int = read()
domain V = 1..=n
domain E = 1..n
domain Node = V | E
let tree = Tree<Node>.subdivide(edges: read<[E](V, V)>())
let pow2 = Powers<Z>(2, n)
let pow3 = Powers<Z>(3, n)
fn some_red(c: Int) -> Z = pow2[c] - 1
fn red_and_blue(c: Int) -> Z = pow3[c] - 2 * pow2[c] + 1

let ans = sum for centre in Node:
    let total = tree.layers(from: centre).map(l => l.count(is V))
    let branches = [
        tree.layers(
            from: c,
            avoiding: centre,
        ).shifted(1).map(l => l.count(is V)) for c in tree[centre]
    ]
    let red = total.map(some_red) - sum(b.map(some_red) for b in branches)
    let both = total.map(red_and_blue) - sum(b.map(red_and_blue) for b in branches)

    // Over the layers inside d: all colourings, those whose outermost blue
    // vertex exists (layers past it red or blank), and their sum of its depth.
    var (inside, blue_inside, blue_depths): (Z, Z, Z) = (1, 0, 0)
    sum for (d, c) in total.items:
        let here = inside * both[d] * (2 * d) + (blue_depths + blue_inside * d) * red[d] * 2
        let outermost_blue = inside * (pow3[c] - pow2[c])
        (blue_inside, blue_depths) = (
            blue_inside * pow2[c] + outermost_blue,
            blue_depths * pow2[c] + outermost_blue * d,
        )
        inside *= pow3[c]
        here
print(ans / 2)

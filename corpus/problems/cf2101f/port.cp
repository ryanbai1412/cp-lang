// Source: Codeforces 2101F, https://codeforces.com/contest/2101/problem/F
// limits: 7000 ms, 256 MB
// Problem: a tree on n <= 3000 vertices (multitest, sum n <= 3000). Colour each vertex red,
//   blue or white; a colouring scores the max distance between a red and a blue vertex (0 if
//   either is missing). Sum the score over all 3^n colourings mod 998244353.
// Solution: subdivide every edge; a colouring's farthest pair then has a centre c with radius
//   d, its coloured vertices within d of c and those at d in >= 2 branches of c. Per centre,
//   DFS each branch for layer sizes; "in >= 2 branches" is the whole layer minus each branch
//   alone. If layer d holds both colours the score is 2d, else it is all red (x2 for blue) and
//   scores d + d' for the outermost blue depth d', summed by a running scan over d. O(n) per
//   centre, O(n^2) total; halve for the subdivision.
// Bounds: n: 1..=3000; u, v: 1..2 * n

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

let pow2 = [0..=n] k => Z(2) ** k
let pow3 = [0..=n] k => Z(3) ** k
fn some_red(c: Int) -> Z = pow2[c] - 1
fn red_and_blue(c: Int) -> Z = pow3[c] - pow2[c] * 2 + 1

let depths = Node.count
var ans: Z = 0
for centre in Node:
    var total: [0..depths]Int = 0
    if is_vertex(centre): total[0] = 1
    var red: [0..depths]Z = 0
    var both: [0..depths]Z = 0
    for c in adj[centre]:
        var branch: [0..depths]Int = 0
        fn count(v: Node, from: Node, depth: Int):
            if is_vertex(v): branch[depth] += 1
            for w in adj[v] if w != from: count(w, v, depth + 1)
        count(c, centre, 1)
        for d in 0..depths:
            total[d] += branch[d]
            red[d] -= some_red(branch[d])
            both[d] -= red_and_blue(branch[d])
    for d in 0..depths:
        red[d] += some_red(total[d])
        both[d] += red_and_blue(total[d])

    // Over the layers inside d: all colourings, those whose outermost blue
    // vertex exists (layers past it red or blank), and their sum of its depth.
    var inside: Z = 1
    var blue_inside: Z = 0
    var blue_depths: Z = 0
    for d in 0..depths:
        let c = total[d]
        ans += inside * both[d] * (2 * d)
        ans += (blue_depths + blue_inside * d) * red[d] * 2
        let outermost_blue = inside * (pow3[c] - pow2[c])
        blue_inside = blue_inside * pow2[c] + outermost_blue
        blue_depths = blue_depths * pow2[c] + outermost_blue * d
        inside *= pow3[c]
print(ans / 2)

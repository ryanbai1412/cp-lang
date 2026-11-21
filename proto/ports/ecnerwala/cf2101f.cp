use lib.math.powers.{Powers}

type Z = Mod<998_244_353>

@! multitest
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
for root in Node:
    var red: [0..Node.K]Z = 0
    var both: [0..Node.K]Z = 0
    var total: [0..Node.K]Int = 0
    var branch: [0..Node.K]Int = 0
    if is_vertex(root): total[0] += 1

    fn dfs(v: Node, from: Node, depth: Int):
        if is_vertex(v):
            total[depth] += 1
            branch[depth] += 1
        for w in adj[v] if w != from:
            dfs(w, v, depth + 1)

    let first = if is_vertex(root): 0 else: 1
    for child in adj[root]:
        dfs(child, root, 1)
        for d in (first..Node.K).step(2):
            red[d] -= some_red(branch[d])
            both[d] -= red_and_blue(branch[d])
            branch[d] = 0

    var below_both: Z = 1
    var only_red: Z = 0
    var red_dist: Z = 0
    for d in (first..Node.K).step(2):
        let c = total[d]
        red[d] += some_red(c)
        both[d] += red_and_blue(c)
        ans += d * 2 * below_both * both[d]
        ans += (red_dist + d * only_red) * red[d] * 2
        only_red *= pow2[c]
        red_dist *= pow2[c]
        let extra = pow3[c] - pow2[c]
        only_red += below_both * extra
        red_dist += d * below_both * extra
        below_both *= pow3[c]
print(ans / 2)

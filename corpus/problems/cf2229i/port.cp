// Source: Codeforces 2229I (The Endians), https://codeforces.com/contest/2229/problem/I
// limits: 2000 ms, 1024 MB
// Problem: a tree of n <= 4000 nodes (sum n <= 4000, multitest) with weights w_i <= 1e9, and
//   k. Rooted at x, choose S of k nodes containing x; f(i) sums the weights of chosen nodes
//   on the path from i to the root, and the score is the sum of f(i) over i in S. Print the
//   best score for every root x.
// Solution: the score is the sum over a in S of w(a) * |S in subtree(a)|. down[v][i], the
//   best total over subtree(v) with i chosen inside, is a max-plus knapsack over the
//   children, then f[i] max= f[i - 1] + w(v) * i for choosing v: O(n^2). Rerooting, up[v][i]
//   is the best total of the nodes outside subtree(v) given i chosen inside; a child's part
//   is removed by the transposed convolution split(f, b)[i] = max_j f[i + j] + b[j] (later
//   siblings, then the stored prefix join of earlier ones, then v itself with k - i chosen
//   outside). With all children split off, up[v][1] + w(v) * k is the answer at root v;
//   up[v][1] is never none, since k <= n leaves room for the k - 1 others outside v.
//   kids go smallest subtree first only for memory: the time is O(n^2) in any order, but
//   the largest child is then in no stored prefix join (a star whose long path came first
//   would copy the path into every leaf's prefix).
@!multitest
let (n, k): (2..=4000, 1..=n) = read()
domain V = 1..=n
let w: [V]1..=1_000_000_000 = read()
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted(Graph.undirected(edges), root: V.first!)
let kids: [V]List<V> = [V] u => t.children(u).sorted(by: c => t.size(c))

fn split(f: List<?Int>, b: List<Int>) -> List<?Int>:
    var res: List<?Int> = [None for _ in 0..f.len - b.len + 1]
    for i in 0..res.len:
        for j in 0..b.len:
            if let x = f[i + j]: res[i] max= x + b[j]
    res

var down: [V]List<Int> = []
var joined_before: [V]List<List<Int>> = []
for v in t.at.rev:
    var f = [0]
    for c in kids[v]:
        joined_before[v].push(f.clone())
        f = convolve(f, down[c], over: Semiring(max, +), shape: Curved.Neither)
    f.push(0)
    for i in (1..f.len).rev: f[i] max= f[i - 1] + w[v] * i
    down[v] = f

var ans: [V]Int = 0
var up: [V]List<?Int> = []
up[t.root] = [if i == k: 0 else: None for i in 0..=n]
for v in t.at:
    var f = up[v]
    for (z, c) in kids[v].items.rev:
        let g = split(f, joined_before[v][z])
        up[c] = [max(g[i], g[i + 1].map(x => x + w[v] * (k - i))) for i in 0..g.len - 1]
        f = split(f, down[c])
    ans[v] = f[1]! + w[v] * k

print(ans)

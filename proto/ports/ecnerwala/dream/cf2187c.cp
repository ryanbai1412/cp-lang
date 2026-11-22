// Codeforces 2187 C — Jerry and Tom
//
// In the DAG with backbone edges i -> i + 1 plus m forward edges u -> v, the
// optimal next move from every non-sink vertex i is the farthest reachable
// vertex nxt[i] = max(i + 1, max_{(i, v)} v) > i. These parent pointers form
// a tree rooted at the sink V.last.
//
// For any pair of starting vertices (x, y) whose LCA in this tree is p, the
// game ends after min(depth(x) - depth(p), depth(y) - depth(p)) steps. To sum
// this over all pairs (x, y) in O(n) via small-to-large merging, each node i
// keeps a reversed depth table `by_depth[i]` where entry `len - l` (l >= 1)
// stores `Bucket(cnt, sum)`: the number of descendants of i at distance l - 1
// below i, and the sum of their subtree sizes (each node u at depth l contributes
// sz[u] descendants at depth >= l).
//
// When merging child i into parent p (after swapping so `into` is the longer
// table), every depth l in the shorter table pairs nodes at depth l in one
// branch with all nodes at depth >= l in the other branch:
//   (a.cnt * b.sum + b.cnt * a.sum) * l.

struct Bucket(cnt: Int, sum: Int)

fn merge_by_depth(mut into: List<Bucket>, from: List<Bucket>) -> Int:
    let na = into.len
    let nb = from.len
    var gained = 0
    for l in 1..=nb:
        let a = into[na - l]
        let b = from[nb - l]
        gained += (a.cnt * b.sum + b.cnt * a.sum) * l
        into[na - l] = Bucket(a.cnt + b.cnt, a.sum + b.sum)
    gained

@!multitest
let (n, m): (2..=200_000, 0..=200_000) = read()
domain V = 1..=n
var nxt: [V]V = [V] i => i.next ?? i
let edges: [0..m](u: V, v: V) = read()
for (u, v) in edges: nxt[u] max= v

var sz: [V]Int = 0
var by_depth: [V]List<Bucket> = []
var ans = 0

for i in V:
    sz[i] += 1
    by_depth[i].push(Bucket(1, sz[i]))
    if i < V.last:
        let p = nxt[i]
        assert p != i
        sz[p] += sz[i]
        if by_depth[p].len < by_depth[i].len:
            swap(mut by_depth[p], mut by_depth[i])
        ans += merge_by_depth(mut by_depth[p], by_depth[i])

print(ans)

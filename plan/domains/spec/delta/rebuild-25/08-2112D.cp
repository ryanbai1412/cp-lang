// Codeforces 2112D Reachability and Tree: https://codeforces.com/problemset/problem/2112/D
// Alternating directions give exactly the n - 1 edge pairs; one extra pair needs exactly one
// directed path of length 2, a -> mid -> b through a vertex of degree 2. Away from mid, a's side
// alternates starting with a as a source, b's side starting with b as a sink.
@!multitest
let n: 2..=2e5 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
let mid = V.find(v => g.deg(v) == 2) else:
    print("NO")
    return
let t = Rooted(g, root: mid)
let kids = List(t.children(mid))
var source: [V]Bool = false
source[kids[0]] = true
print("YES")
print(kids[0], mid)
print(mid, kids[1])
for v in t.order:
    let p = t.parent(v) else: continue
    if p == mid: continue
    source[v] = not source[p]
    if source[p]: print(p, v) else: print(v, p)

@check
fn ok():
    let n: Int = read()
    let edges = read<[n - 1](u: Int, v: Int)>()
    var deg: [n + 1]Int = 0
    for e in edges:
        deg[e.u] += 1
        deg[e.v] += 1
    let word: Str = read_output()
    if word == "NO":
        assert not (1..=n).any(v => deg[v] == 2), "said NO, but a vertex has degree 2"
        return
    assert word == "YES", "want YES or NO, got {word}"
    var out = [n + 1] _ => List<Int>()
    var seen: Set<(Int, Int)> = Set()
    repeat n - 1:
        let (u, v): (Int, Int) = read_output()
        assert 1 <= u <= n and 1 <= v <= n, "vertex out of range"
        seen.add((min(u, v), max(u, v)))
        out[u].push(v)
    assert edges.all(e => (min(e.u, e.v), max(e.u, e.v)) in seen), "edges differ from the tree"
    var pairs = 0
    for s in 1..=n:
        var reach: [n + 1]Bool = false
        var stack = [s]
        reach[s] = true
        while let u = stack.pop():
            for v in out[u] if not reach[v]:
                reach[v] = true
                pairs += 1
                stack.push(v)
    assert pairs == n, "{pairs} good pairs, want {n}"

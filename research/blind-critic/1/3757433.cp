// Corpus submission 3757433: tree pebbling construction (enum Op + match + recursive DFS)
let n: 2..=1000 = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()

var adj: [V]List<V> = []
for e in E:
    let (u, v) = edges[e]
    adj[u].push(v)
    adj[v].push(u)

enum Op:
    L(u: V)
    B(u: V)
    M(u: V, v: V)

fn tdfs(u: V, p: ?V) -> Int:
    let deg = adj[u].len - Int(p != none)
    if deg == 1:
        var nxt = adj[u][0]
        if let pv = p:
            if nxt == pv:
                nxt = adj[u][1]
        return tdfs(nxt, u)
    if adj[u].len == 1:
        return 1
    var best = 0
    for v in adj[u]:
        if p == none or v != p!:
            best max= tdfs(v, u)
    1 + best

var ans: List<Op> = []
var mans = 0
var ccur = 0

fn mk(u: V):
    ans.push(L(u))
    ccur += 1
    mans max= ccur

fn rm(u: V):
    ans.push(B(u))
    ccur -= 1

fn mv(u: V, v: V):
    ans.push(M(u, v))

fn dfs(u: V, p: ?V):
    let deg = adj[u].len - Int(p != none)
    if deg == 1:
        var nxt = adj[u][0]
        if let pv = p:
            if nxt == pv:
                nxt = adj[u][1]
        dfs(nxt, u)
        mv(nxt, u)
        return
    if adj[u].len == 1:
        mk(u)
        mv(u, p!)
        return
    mk(u)
    for v in adj[u]:
        if p == none or v != p!:
            dfs(v, u)
            rm(u)
    if let pv = p:
        mv(u, pv)

var best_cost: ?Int = none
var best_root: ?V = none
for v in V:
    let c = tdfs(v, none)
    if best_cost == none or c < best_cost!:
        best_cost = c
        best_root = v

dfs(best_root!, none)
print(mans)
print(ans.len)
for op in ans:
    match op:
        L(u):
            print("L", u)
        B(u):
            print("B", u)
        M(u, v):
            print("M", u, v)

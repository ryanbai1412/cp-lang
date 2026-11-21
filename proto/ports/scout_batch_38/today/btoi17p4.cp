// Baltic OI '17 P4 (Friends): bounded search tree + submodular cut uncrossing.
// Runnable on proto/cpc today.
// original: corpus/submissions/4523876.cpp
let n: 1..=2500 = read()
let p: 1..=16 = read()
let q: 0..=16 = read()
domain V = 0..n
var adj: [V]List<V> = []
var bad = false
for u in V:
    let deg: 0..=n = read()
    if deg > p + q - 1: bad = true
    for _ in 0..deg:
        let v: V = read()
        adj[u].push(v)

var decided: [V]Int = 0
var incl: List<V> = []
var cans: List<V> = []
var found = false

fn dfs():
    if found or incl.len > p: return
    var cq = 0
    for idx in 0..incl.len:
        let a = incl[idx]
        for v in adj[a]:
            if decided[v] == 0:
                decided[v] = 1
                incl.push(v)
                dfs()
                incl.pop()
                decided[v] = -1
                dfs()
                decided[v] = 0
                return
            elif decided[v] == -1:
                cq += 1
                if cq > q: return
    cans = []
    for x in incl: cans.push(x)
    found = true

fn solve():
    if bad:
        print("detention")
        return
    for u in V:
        for v in adj[u]:
            var ok = false
            for w in adj[v]:
                if w == u:
                    ok = true
                    break
            if not ok:
                print("detention")
                return

    var blocks: [V]List<V> = []
    for u in V:
        incl = [u]
        decided[u] = 1
        found = false
        dfs()
        decided[u] = 0
        if not found:
            print("detention")
            return
        cans.sort()
        blocks[u] = cans

    var vis: [V]Int = 0
    var vind = 0

    fn excl(a: List<V>, b: List<V>) -> List<V>:
        vind += 1
        for x in b: vis[x] = vind
        var out: List<V> = []
        for x in a:
            if vis[x] != vind: out.push(x)
        out

    fn valid(arr: List<V>) -> Bool:
        if arr.len > p: return false
        vind += 1
        for x in arr: vis[x] = vind
        var cut = 0
        for u in arr:
            for v in adj[u]:
                if vis[v] != vind:
                    cut += 1
                    if cut > q: return false
        true

    for i in 0..n:
        for j in i + 1..n:
            let x = excl(blocks[V(i)], blocks[V(j)])
            if valid(x):
                blocks[V(i)] = x
            else:
                blocks[V(j)] = excl(blocks[V(j)], blocks[V(i)])

    var non_empty: List<V> = []
    for u in V:
        if blocks[u].len > 0: non_empty.push(u)

    print("home")
    print(non_empty.len)
    for u in non_empty:
        var line: List<Int> = [blocks[u].len]
        for x in blocks[u]: line.push(x.idx)
        print(line)

solve()

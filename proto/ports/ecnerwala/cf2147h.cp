@!multitest
let n: 1..=50 = read()
let m: 0..=1225 = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V, w: 1..=1_000_000) = read()

var fg = FlowGraph<V>()
for e in edges: fg.add_both(e.u, e.v, e.w)
var par: [V]V = V.first
var g = 0
for i in V if i != V.first:
    fg.reset()
    g = gcd(g, fg.max_flow(i, par[i]))
    let cut = fg.min_cut(i)
    for j in V.after(i) if par[j] == par[i] and cut[j]:
        par[j] = i

if g != 1:
    print(1)
    print(n)
    print([v for v in V])
else:
    var adj: [V]Bitset<V> = [V] _ => Bitset<V>.new()
    for e in edges if e.w % 2 == 1:
        adj[e.u].toggle(e.v)
        adj[e.v].toggle(e.u)

    fn flip_clique(nbr: Bitset<V>):
        for u in nbr.each():
            adj[u] ^= nbr
            adj[u].remove(u)

    var alive = Bitset<V>.full()
    var stk: List<V> = []
    while let v = first(v for v in alive.each() if (adj[v] & alive).len % 2 == 1):
        alive.remove(v)
        stk.push(v)
        flip_clique(adj[v] & alive)

    var side1 = Bitset<V>.new()
    while let v = stk.pop():
        let nbr = adj[v] & alive
        flip_clique(nbr)
        alive.add(v)
        if (nbr & side1).len % 2 == 0: side1.add(v)

    let (c1, c0) = [v for v in V].partition(v => v in side1)
    print(2)
    print(c0.len)
    print(c0)
    print(c1.len)
    print(c1)

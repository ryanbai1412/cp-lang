// original: 5127828 (noip19p2)
// Rethought: the original keeps a segment tree over the root path; a stack of unmatched '('
// on the path plus f[u] = f[parent(match)] + 1 does the same in O(n).
let n: Int = read()
domain V = 1..=n
let s: [V]Char = read()
var par: [V]?V = none
var ch: [V]List<V> = []
for v in V.first.r..:
    let p: V = read()
    par[v] = p
    ch[p].push(v)
var f: [V]Int = 0
var open: List<V> = []
var ans = 0
fn dfs(u: V, acc: Int):
    var popped: ?V = none
    if s[u] == '(': open.push(u)
    else:
        popped = open.pop()
        if let o = popped:
            f[u] = 1
            if let p = par[o]: f[u] += f[p]
    let tot = acc + f[u]
    ans ^= (u.to_idx + 1) * tot
    for v in ch[u]: dfs(v, tot)
    if s[u] == '(': open.pop()
    elif let o = popped: open.push(o)
dfs(V.first, 0)
print(ans)

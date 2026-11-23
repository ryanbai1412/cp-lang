// Codeforces 1984 E, tuned: orig.cpp's two passes over the BFS order with the
// sibling exclusion done by subtracting (counts and taken-flags are sums), in
// place of `reroot`'s per-vertex prefix list. down[v] is the greedy count of
// v's subtree and whether v itself is taken; a leaf r rooted at the top scores
// the count of everything above it plus one.
@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](V, V)>())
let root = V.first!
let t = Rooted(g, root)
let order = t.order

var cnt: [V]Int = 0
var taken: [V]Int = 0
for v in order.rev:
    var (c, k) = (0, 0)
    for ch in t.children(v):
        c += cnt[ch]
        k += taken[ch]
    taken[v] = Int(k == 0)
    cnt[v] = c + taken[v]

var up_cnt: [V]Int = 0
var up_taken: [V]Int = 0
var ans = 0
if g[root].len == 1: ans = cnt[g[root][0]] + 1
for v in order:
    var (c, k) = (up_cnt[v], up_taken[v])
    for ch in t.children(v):
        c += cnt[ch]
        k += taken[ch]
    if v != root and g[v].len == 1: ans max= c + 1
    for ch in t.children(v):
        let free = Int(k - taken[ch] == 0)
        up_cnt[ch] = c - cnt[ch] + free
        up_taken[ch] = free
print(ans)

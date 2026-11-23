// Tuned: trie.cp with the node list reserved up front, the query loop
// `unchecked:`.
const L = 25

struct Trie:
    var cnt: Int
    var kid: [0..2]?Int

var nodes = List<Trie>()

fn merge(x: ?Int, y: ?Int) -> ?Int:
    if x == None: return y
    if y == None: return x
    nodes[x].cnt += nodes[y].cnt
    for b in 0..2: nodes[x].kid[b] = merge(nodes[x].kid[b], nodes[y].kid[b])
    x

fn chain(x: Int) -> Int:
    nodes.push(Trie(1, [0..2] _ => None))
    for z in 0..L:
        let below = nodes.len - 1
        let bit = Int(x.bit(z))
        nodes.push(Trie(1, [0..2] b => if b == bit: below else: None))
    nodes.len - 1

fn query_trie(root: Int, mask: Int, want: Int) -> Int:
    var (t, total) = (root, 0)
    unchecked:
        for b in (0..L).rev:
            if let next = nodes[t].kid[(want >> b) & 1]:
                t = next
                if mask.bit(b): total += nodes[t].cnt << b
            else: break
    total

let n = 25_000
var x = 123456789
var a = List<Int>()
for _ in 0..n:
    x = (x * 1103515245 + 12345) & ((1 << 31) - 1)
    a.push((x >> 6) & ((1 << L) - 1))

nodes.reserve((L + 1) * n)
var groups = List<(mask: Int, root: Int)>()
var (pref, ans) = (0, 0)
for v_in in a:
    groups.push((mask: (1 << L) - 1, root: chain(pref)))
    var kept = List<(mask: Int, root: Int)>()
    for g in groups:
        let v = g.mask & v_in
        if let k = kept.last and k.mask == v: merge(k.root, g.root)
        else: kept.push((mask: v, root: g.root))
    groups = kept
    pref ^= v_in
    for g in groups: ans += query_trie(g.root, g.mask, pref ^ g.mask)
print(ans)

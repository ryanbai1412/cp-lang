// Codeforces 2261 G: sum over subarrays of (xor of the subarray) AND (and of
// the subarray). For a fixed right end the suffix ANDs take at most 25 values;
// each keeps a binary trie of the prefix xors of its left ends, and tries are
// merged when two suffix ANDs become equal.
const L = 25

struct Trie(cnt: Int, kid: [0..2]?Int)

@!multitest
let n: 1..=2e5 = read()
let a: [0..n]0..(1 << L) = read()
var nodes = List<Trie>()

fn merge(x: ?Int, y: ?Int) -> ?Int:
    if x == none: return y
    if y == none: return x
    nodes[x].cnt += nodes[y].cnt
    for b in 0..2: nodes[x].kid[b] = merge(nodes[x].kid[b], nodes[y].kid[b])
    x

// A path of L + 1 nodes spelling `x` from the top bit down; returns its root.
fn chain(x: Int) -> Int:
    nodes.push(Trie(1, [0..2] _ => none))
    for z in 0..L:
        let below = nodes.len - 1
        let bit = Int(x.bit(z))
        nodes.push(Trie(1, [0..2] b => if b == bit: below else: none))
    nodes.len - 1

fn query_trie(root: Int, mask: Int, want: Int) -> Int:
    var (t, total) = (root, 0)
    for b in (0..L).rev:
        if let next = nodes[t].kid[Int(want.bit(b))]:
            t = next
            if mask.bit(b): total += nodes[t].cnt << b
        else: break
    total

var groups = List<(mask: Int, root: Int)>()  // distinct suffix ANDs, each with its trie
var (pref, ans) = (0, 0)
for x in a:
    groups.push((mask: (1 << L) - 1, root: chain(pref)))
    var kept = List<(mask: Int, root: Int)>()
    for g in groups:
        let v = g.mask & x
        if let k = kept.last and k.mask == v: merge(k.root, g.root)
        else: kept.push((mask: v, root: g.root))
    groups = kept
    pref ^= x
    for g in groups: ans += query_trie(g.root, g.mask, pref ^ g.mask)
print(ans)

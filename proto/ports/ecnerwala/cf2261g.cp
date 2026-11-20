// Codeforces 2261 G: sum over subarrays of (xor of the subarray) AND (and of
// the subarray). For a fixed right end the suffix ANDs take at most 25 values;
// each keeps a binary trie of the prefix xors of its left ends, and tries are
// merged when two suffix ANDs become equal.
const L = 25
type Trie = (cnt: Int, kid: (?Int, ?Int))
@!multitest
let n: 1..=2e5 = read()
let a: [0..n]0..(1 << L) = read()
var nodes = List<Trie>()

fn merge(x: ?Int, y: ?Int) -> ?Int:
    if x == none: return y
    if y == none: return x
    let (p, q) = (x!, y!)
    nodes[p].cnt += nodes[q].cnt
    nodes[p].kid = (merge(nodes[p].kid.0, nodes[q].kid.0), merge(nodes[p].kid.1, nodes[q].kid.1))
    x

// A path of L + 1 nodes spelling `x` from the top bit down; returns its root.
fn chain(x: Int) -> Int:
    nodes.push((cnt: 1, kid: (none, none)))
    for z in 0..L:
        let below = nodes.len - 1
        nodes.push((cnt: 1, kid: if x >> z & 1 == 0: (below, none) else: (none, below)))
    nodes.len - 1

var groups = List<(mask: Int, root: Int)>()    // distinct suffix ANDs, each with its trie
var (pref, ans) = (0, 0)
for x in a:
    groups.push((mask: (1 << L) - 1, root: chain(pref)))
    var kept = List<(mask: Int, root: Int)>()
    for g in groups:
        let v = g.mask & x
        if kept.len > 0 and kept.last!.mask == v: merge(kept.last!.root, g.root)
        else: kept.push((mask: v, root: g.root))
    groups = kept
    pref ^= x
    for g in groups:
        let want = pref ^ g.mask
        var t: ?Int = g.root
        for b in (0..L).rev:
            let kid = nodes[t!].kid
            t = if want >> b & 1 == 0: kid.0 else: kid.1
            if t == none: break
            if g.mask >> b & 1 == 1: ans += nodes[t!].cnt << b
print(ans)

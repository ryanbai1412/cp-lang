// Codeforces 2261 G: sum over subarrays of (xor of the subarray) AND (and of
// the subarray). For a fixed right end the suffix ANDs take at most 25 values;
// each keeps a binary trie of the prefix xors of its left ends, and tries are
// merged when two suffix ANDs become equal. A group with AND `mask` adds, for
// each set bit b of `mask`, 2^b times the number of its left ends whose prefix
// xor differs from the current one at b: walking the trie along
// `pref ^ mask` counts exactly those as it passes each bit of `mask`.
const L = 25
const FULL = (1 << L) - 1

struct Node:
    var cnt: Int
    kid: [Bool]?Int
struct Group(mask: Int, root: Int)

@!multitest
let n: 1..=2e5 = read()
let a: [0..n]0..=FULL = read()
var nodes = List<Node>()

fn merge(x: Int, y: Int) -> Int:
    nodes[x].cnt += nodes[y].cnt
    for b in [false, true] if let c = nodes[y].kid[b]:
        nodes[x].kid[b] = if let k = nodes[x].kid[b]: merge(k, c) else: c
    x

// A path of L + 1 nodes spelling `x` from the top bit down; returns its root.
fn chain(x: Int) -> Int:
    nodes.push(Node(1, [Bool] _ => None))
    for z in 0..L:
        let below = nodes.len - 1
        var node = Node(1, [Bool] _ => None)
        node.kid[x.bit(z)] = below
        nodes.push(node)
    nodes.len - 1

fn count_along(root: Int, mask: Int, want: Int) -> Int:
    var (t, total) = (root, 0)
    for b in (0..L).rev:
        guard let next = nodes[t].kid[want.bit(b)] else: break
        t = next
        if mask.bit(b): total += nodes[t].cnt << b
    total

var groups = List<Group>()
var (pref, ans) = (0, 0)
for x in a:
    groups.push(Group(FULL, chain(pref)))
    var kept = List<Group>()
    for g in groups:
        let v = g.mask & x
        if let k = kept.last and k.mask == v: kept[kept.len - 1] = Group(v, merge(k.root, g.root))
        else: kept.push(Group(v, g.root))
    groups = kept
    pref ^= x
    ans += sum(count_along(g.root, g.mask, pref ^ g.mask) for g in groups)
print(ans)

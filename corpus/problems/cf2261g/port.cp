// Source: Codeforces 2261G (1 2 3 and XOR, Where Did Everybody Go?),
//   https://codeforces.com/contest/2261/problem/G
// Problem: f(b) is the least b_1 xor ... xor b_k after adding any non-negative number to
//   one element of b. Given a_1..a_n (a_i < 2^25), print the sum of f over all subarrays.
//   1 <= n <= 4e5, t <= 1e4 tests, sum n <= 4e5.
// Solution: the increment turns b_i into any c >= b_i, and the xor becomes R ^ c with
//   R ^ b_i = X, the whole xor. Down X's bits from the top, c must copy b_i until the first X
//   bit b_i lacks (there c goes above b_i and then copies R), so f keeps the top X bits that
//   every element has, i.e. that the AND has. Each AND bit's X bit is the length's parity, so
//   f sums 2^b over the AND bits b with X == AND on all bits >= b. For a fixed right end
//   the suffix ANDs take <= 25 values; each group keeps a binary trie of the prefix xors of
//   its left ends, merged when two ANDs coincide. Walking a group's trie along pref ^ mask
//   counts, at each set bit b of mask, the left ends whose XOR agrees with the AND on bits
//   >= b; add 2^b times that. O(n * 25^2) plus trie merges.
const L = 25
const FULL = (1 << L) - 1

struct Node:
    var cnt: Int
    kid: [Bool]?Int
struct Group(mask: Int, root: Int)

@!multitest
let n: 1..=4e5 = read()
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

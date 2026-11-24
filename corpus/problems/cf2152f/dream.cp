// Codeforces 2152 F. The greedy that starts at point i next takes the first
// point more than z to the right. That is a tree on 0..=n rooted at n (the
// walk has left the points), and every parent lies to the right, so the tree
// grows leaf by leaf from right to left.
//
// The answer for [l, r) runs the greedy from l and from l + 1 side by side:
// the second walk is the one that skips the first point. The two walks meet at
// their lca c and from there continue exactly as the pair (c, c + 1), so pairs
// form a second tree. `taken[i]` counts the points the walks from i and i + 1
// take before they meet, summed up that tree: it is the tree distance from i
// to i + 1, plus `taken[c]`. A query climbs the pair tree while it stays left
// of r, then finishes each walk of the last pair in the greedy tree.

@!multitest
let (n, z) = read<(Int, Int)>()
let x: [0..n]Int = read()
domain V = 0..=n
var (greedy, pairs) = (JumpTree<V>(), JumpTree<V>())
var taken: [V]Int = 0
for i in V.rev if i != V.last:
    greedy.add(i, V((i.ord + 1..=n).first_true(j => j == n or x[j] > x[i.ord] + z)!))
    let meet = greedy.lca(i, i.next!)!
    pairs.add(i, meet)
    taken[i] = taken[meet] + greedy.dist(i, i.next!)!

for (l, r) in read<[0..read()](l: V, r: V)>():
    let l = l.prev!
    let c = pairs.jump_while(l, i => i < r)!
    let finish = sum(
        greedy.dist(st, greedy.jump_while(st, i => i < r)!)! + 1 for st in [c, c.next!] if st < r
    )
    print(taken[l] - taken[c] + finish)

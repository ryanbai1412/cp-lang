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
let greedy = JumpTree<V>.grown(
    V.rev,
    parent: i => V.after(i).first(j => j == V.last or x[j] > x[i] + z),
)
let meet = [V.until_last] i => greedy.lca(i, i.next!)
let pairs = JumpTree<V>.grown(V.until_last.rev, parent: i => meet[i])
let taken = pairs.down(0, (above, i) => above + greedy.dist(i, i.next!))

for (l, r) in read<[0..read()](l: V, r: V)>():
    let l = l.prev!
    let c = pairs.lift_while(l, i => i < r)
    let finish = sum(
        greedy.dist(st, greedy.lift_while(st, i => i < r)) + 1 for st in [c, c.next!] if st < r
    )
    print(taken[l] - taken[c] + finish)

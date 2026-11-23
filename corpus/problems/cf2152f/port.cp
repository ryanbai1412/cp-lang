// Codeforces 2152 F. The greedy that starts at point i next takes the first
// point more than z to the right: a tree on 0..=n, rooted at the sentinel n,
// whose parents lie to the right, so it grows leaf by leaf from right to left.
// The answer for [l, r) runs the greedy from l and from l + 1 side by side.
// The two walks meet at their lca c and continue as the pair (c, c + 1), so
// pairs form a second tree; `taken[i]` counts the points the walks from i and
// i + 1 take up to n. A query climbs the pair tree while it stays left of r,
// then finishes each walk of the last pair in the greedy tree.

@!multitest
let (n, z) = read<(Int, Int)>()
let x: [0..n]Int = read()
domain V = 0..=n
var (greedy, pairs) = (JumpTree<V>(), JumpTree<V>())
fn dist_to(u: V, anc: V) -> Int = greedy.depth(u) - greedy.depth(anc)
fn finish_walk(st: V, r: V) -> Int = dist_to(st, greedy.jump_while(st, i => i < r)!) + 1

var taken: [V]Int = 0
for i in V.rev if i != V.last:
    greedy.add(i, V((i.ord + 1..=n).first_true(j => j == n or x[j] > x[i.ord] + z)!))
    let c = greedy.lca(i, i.next!)!
    pairs.add(i, c)
    taken[i] = taken[c] + dist_to(i, c) + dist_to(i.next!, c)

for (l, r) in read<[0..read()](l: V, r: V)>():
    let l = l.prev!
    let c = pairs.jump_while(l, i => i < r)!
    let finish = sum(finish_walk(st, r) for st in [c, c.next!] if st < r)
    print(taken[l] - taken[c] + finish)

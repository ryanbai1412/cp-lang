// Source: Codeforces 2152F (Triple Attack), https://codeforces.com/contest/2152/problem/F
// limits: 3000 ms, 1024 MB
// Problem: sorted timestamps x_1..x_n (n <= 2.5e5) and z; a set is safe if every three of its
//   elements span more than z. For q <= 2.5e5 ranges [l, r], print the size of the largest
//   safe subset of x_l..x_r. Multitest, sums of n and q <= 2.5e5.
// Solution: the greedy takes l and l + 1, then each next point is the first one more than z
//   past the pick two before. Its odd and even picks are walks in the tree with parent(i) = the
//   first j with x_j > x_i + z (sentinel n) until they meet at their lca c, then go on as the
//   pair (c, c + 1); pairs form a second tree and taken[i] counts the picks from (i, i + 1) to
//   n. A query climbs the pair tree while below r, then finishes both walks; both trees are
//   JumpTrees: O((n + q) log n).

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

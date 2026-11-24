// Source: Codeforces 2152G (Query Jungle), https://codeforces.com/contest/2152/problem/G
// limits: 3000 ms, 1024 MB
// Problem: a tree of n <= 2.5e5 vertices rooted at 1, each with or without a monster. Print the
//   fewest paths from the root that together visit every monster, initially and after each of
//   q <= 2.5e5 queries that toggle the monster state of every vertex in v's subtree. Multitest.
// Solution: in preorder, a monster ends a path iff the next monster in preorder is not in its
//   subtree, so the answer is the number of monsters minus consecutive (preorder) monster pairs
//   where the first is an ancestor of the second. A SegTree over preorder keeps (ends, first,
//   last) per segment for both the monster set and its complement (`Flip`); a subtree query flips
//   one preorder range, swapping the two sides. O((n + q) log n).
// Bounds: n: 2..=250_000; flag: 0..=1

@!multitest
use std.lazy.{Flip, flip_action}
let n: 2..=250_000 = read()
domain V = 1..=n
let flag: [V]0..=1 = read()
let monster = [V] v => flag[v] == 1
let t = Rooted(Graph.undirected(read<[n - 1](V, V)>()), V.first!)

type Side = (ends: Int, first: ?V, last: ?V)
let empty_side: Side = (0, None, None)

fn join(x: Side, y: Side) -> Side:
    var ends = x.ends + y.ends
    if let u = x.last, let w = y.first, t.is_anc(u, w): ends -= 1
    (ends, x.first ?? y.first, y.last ?? x.last)

let side = Monoid<Side>(empty_side, join)
let toggle = flip_action<Side>(side)
var st = SegTree(
    toggle.over,
    t.reorder([V] v => Flip<Side>.of(monster[v], (1, Some(v), Some(v)), empty_side)),
    act: toggle,
)

print(st.all.now.ends)
repeat read():
    st.apply(t.sub(read<V>()), true)
    print(st.all.now.ends)

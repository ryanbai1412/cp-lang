// Source: Codeforces 2062E2 (The Game, hard), https://codeforces.com/contest/2062/problem/E2
// Problem: a tree of n <= 4e5 nodes rooted at 1 with values w_i <= n. Players alternately delete
//   the subtree of a remaining node heavier than the opponent's last pick (Cirno's first pick is
//   free); whoever cannot move wins. Print how many first picks win for Cirno and list them.
//   Multitest, sum n <= 4e5.
// Solution: u wins iff some heavier node lies outside sub(u) and every heavier reply x outside it
//   strands Cirno. With above(x) = nodes heavier than x outside sub(x) and g(x) their lca, x
//   strands her iff above(x) is empty or g(x) is in sub(u). Sweep weights downward: a SortedSet of
//   heavier preorder positions gives g(x) (lca of the first and last outside sub(x)); Fenwicks
//   over preorder mark x, g(x) and lca(x, g(x)) so inclusion-exclusion counts dangerous replies
//   with both outside sub(u). O(n log n).

use std.seq.{buckets}
enum Mark: Reply | Target | Meet

@!multitest
let n: 1..=400_000 = read()
domain V = 1..=n
domain W = 1..=n
let w: [V]W = read()
let t = Rooted.new(Graph.undirected(read<[0..n - 1](V, V)>()), V.first!)

var heavier = SortedSet<t.Pre>()
fn above_lca(x: V) -> ?V:
    let r = t.sub(x)
    let (before, after) = (heavier.range(..r.l), heavier.range(r.r..))
    let lo = (before.first ?? after.first) ?? return None
    let hi = (after.last ?? before.last)!
    Some(t.lca(t.at[lo], t.at[hi]))

var marks = [Mark] _ => Fenwick<t.Pre, Int>.new()
var dangerous = 0
fn inside(m: Mark, u: V) -> Int = marks[m].fold(t.sub(u))

let by_w = buckets(w)
var winners: List<V> = []
for weight in W.rev:
    let bucket = by_w[weight]
    for u in bucket:
        let r = t.sub(u)
        let in_sub = marks[Mark.Reply].fold(r)
        let replies = heavier.len - heavier.range(r).len
        let fatal = dangerous - in_sub - inside(Mark.Target, u) + inside(Mark.Meet, u)
        if replies > 0 and fatal == 0:
            winners.push(u)
    for x in bucket:
        if let g = above_lca(x):
            dangerous += 1
            marks[Mark.Reply].add(t.pos(x), 1)
            marks[Mark.Target].add(t.pos(g), 1)
            marks[Mark.Meet].add(t.pos(t.lca(x, g)), 1)
    for v in bucket:
        heavier.add(t.pos(v))

winners.sort()
print(winners.len, winners)

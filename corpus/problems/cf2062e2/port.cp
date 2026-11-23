// Codeforces 2062 E2: The Game (Hard Version).
//
// Cirno opens with u; Daiyousei must answer with a heavier x outside sub(u),
// and if he can't, he wins. Cirno wins iff such an x exists and every one of
// them leaves her stuck: nothing heavier than x outside sub(u) and sub(x).
// Deeper play never changes the outcome (a game-tree brute force agrees for
// every tree with n <= 9).
//
// Let above(x) be the nodes heavier than x outside sub(x), and g(x) their
// lca. A reply x strands Cirno iff above(x) is empty or sub(u) contains
// g(x). Call x dangerous when above(x) is nonempty. So u wins iff some
// heavier node lies outside sub(u) and no dangerous heavier x has both x and
// g(x) outside sub(u). Sweep weights downward: sub(u) is a preorder range,
// "x and g(x) both inside" is "lca(x, g(x)) inside", and inclusion-exclusion
// turns the condition into three range counts. O(n log n).

use std.seq.{buckets}
enum Mark: Reply | Target | Meet

@!multitest
let n: 1..=400_000 = read()
domain V = 1..=n
domain W = 1..=n
let w: [V]W = read()
let t = Rooted.new(Graph.undirected(read<[0..n - 1](V, V)>()), V.first)

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

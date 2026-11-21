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
with io(base: 1)

@!multitest
domain V = read(1..=4e5)
let w: [V]1..=V.count = read()
let t = Rooted(Graph.undirected(read<[V.count - 1](V, V)>()), root: V.first)
type Pre = t.Pre

var heavier = SortedSet<Pre>()
fn above_lca(x: V) -> ?V:
    let r = t.sub(x)
    let (before, after) = (heavier.range(..r.lo), heavier.range(r.hi..))
    let lo = (before.first ?? after.first)?
    let hi = (after.last ?? before.last)!
    t.lca(t.at(lo), t.at(hi))

enum Mark: Reply, Target, Meet
var marks = [Mark] _ => Fenwick<Pre, Int>()
var dangerous = 0
fn inside(m: Mark, u: V) -> Int = marks[m].sum(t.sub(u))

var winners: List<V> = []
for bucket in V.group(by: v => w[v]).rev:
    for u in bucket:
        let replies = heavier.len - heavier.range(t.sub(u)).len
        let fatal = dangerous - inside(Reply, u) - inside(Target, u) + inside(Meet, u)
        if replies > 0 and fatal == 0:
            winners.push(u)
    for x in bucket:
        if let g = above_lca(x):
            dangerous += 1
            for (m, v) in [(Reply, x), (Target, g), (Meet, t.lca(x, g))]:
                marks[m].add(t.pos(v), 1)
    for v in bucket:
        heavier.add(t.pos(v))

winners.sort()
print(winners.len, winners)

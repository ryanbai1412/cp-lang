// cco20p4: every pair of n towns is joined by a red or a blue road. One colour connects all
// towns; tour a spanning tree of it, a closed walk, and print it rotated to start at each town.
let n = read<2..=2_000>()
domain V = 0..n
let road = read<[Pair<V>]Char>()
fn red(u: V, v: V) -> Bool = road[u, v] == 'R'
fn in_colour(c: Bool) = (u: V) => V.filter(v => v != u and red(u, v) == c)
let c = reachable(in_colour(true), from: [V.first!]).count() == n
var walk = List(Rooted(root: V.first!, next: in_colour(c)).euler_tour())
walk.pop()
for v in V:
    print(walk.len)
    print(walk.rotated_left(walk.keys.first(i => walk[i] == v)!))

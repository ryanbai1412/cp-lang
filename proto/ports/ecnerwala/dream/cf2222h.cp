// Codeforces 2222 H: Counting Sort?
// Read an array x as the partition with x_v parts equal to v; then f(x) reads as T(x),
// the partition of its multiplicities, and g(a) counts the partitions on a's T-orbit.
// T(p) has as many cells as p has parts, so T shrinks every p but 1^q -> (q) -> (1):
// its only cycles are the fixed points () and (1). So a and f(a) never recur unless
// a = 0 or a is one 1 among zeros, and g(a) = 2 + orbit(T(T(a))), a partition of at
// most n: the multiplicities of a's nonzero values. Count arrays by it, placing
// v = n down to 1 into the free positions with r_i >= v. About 1.3e6 partitions of
// at most 50 and 4e8 transitions at n = 50, inside the 8 s limit.
type Z = Mod<998_244_353>

@!multitest
let (n, k): (1..=50, 1..=1000) = read()
domain Pos = 1..=n
let r: [Pos]0..=n = read()
let comb = Comb<Z>(n)

domain Shape = Partitions(upto: n)
fn T(p: Shape) -> Shape = Shape.of(p.parts.runs().map(run => run.len))
@memo fn orbit(p: Shape) -> Int = if T(p) == p: 1 else: 1 + orbit(T(p))

var ways: [Shape]Z = 0
ways[Shape.empty] = 1
for v in (1..=n).rev:
    let free = r.count(x => x >= v)
    var placed: [Shape]Z = 0
    for p in Shape if ways[p] != 0:
        let left = free - p.size
        for x in 0..=left: placed[p.with(x)] += ways[p] * comb.choose(left, x)
    ways = placed

var count = Map<Int, Z>()
for p in Shape if p != Shape.empty: count[2 + orbit(p)] += ways[p]
count[1] += 1                                        // a = 0
for i in Pos if r[i] >= 1:                           // a single 1 at i maps to e_1, fixed
    count[3] -= 1
    count[if i == Pos.first: 1 else: 2] += 1
print([count.get(g) ?? 0 for g in 1..=k])

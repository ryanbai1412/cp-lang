// CCO 2019 Marshmallow Molecules: marshmallows 1..n, some joined by sticks.
// Going up from the lowest, whenever a marshmallow is joined to two higher
// ones, those two get a stick as well. Count the sticks at the end.
//
// Only the lowest higher neighbour `f` of v matters: the others all end up
// joined to f, and f passes them on in its own turn. So v hands its set to
// f, and merging the smaller set into the larger keeps it O(n log^2 n).
let (n, m): (Int, Int) = read()
domain V = 1..=n
domain Stick = 0..m
let sticks: [Stick](lo: V, hi: V) = read()

var above = [V] v => Set<V>()  // the higher neighbours of v
for (lo, hi) in sticks: above[lo].add(hi)

var total = 0
for v in V:
    total += above[v].len
    if above[v].len == 0: continue
    let f = above[v].min()
    above[v].remove(f)
    if above[f].len < above[v].len: swap(mut above[f], mut above[v])
    for x in above[v]: above[f].add(x)
print(total)

// coci21c4p4: a read edge list of anonymous records keeps each field at its bound, 12 B an edge
// as in the matched C++'s tuple<int, int, short>.
let n: 1..=4_000_000 = read()
let roads: [0..n](u: 1..=n, v: 1..=n, w: 1..=1000) = read()
var s = 0
for i in 0..n: s += roads[i].u * 3 + roads[i].v + roads[i].w
print(s)

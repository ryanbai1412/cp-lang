// TSP: n <= 16 cities and a distance matrix; the length of the shortest tour that
// starts at city 0, visits every city once and returns. best[s, v]: 0 -> v through s.
let n = read<2..=16>()
domain C = 0..n
let d = read<[C, C]0..=1e6>()
var best: [C.subsets, C]?Int = None
best[Subset<C>.single(0), 0] = 0
for s in C.subsets, v in C if v in s:
    let cur = best[s, v] else: continue
    for u in C if not (u in s):
        best[s.with(u), u] min= cur + d[v, u]
var tour: ?Int = None
for v in C if v != 0:
    tour min= best[Subset<C>.full, v] + d[v, 0]
print(tour!)

// CCO 2021 "Bread First Search": add the fewest edges so that 0, 1, .., n-1
// is a BFS order from 0. Layers are consecutive blocks. A block starting at i
// pays one edge per vertex in it whose earliest neighbour is not before i, and
// the next block may start at cut c only if no vertex from c on has an earliest
// neighbour before i (it would have been found sooner).
let (n, m) = read<(1..=2e5, 0..=2e5)>()
domain V = [n]
var earliest: [V]?V = []
for e in read<[m](u: V, v: V)>():
    let (lo, hi) = minmax(e.u, e.v)
    earliest[hi] = min(earliest[hi], Some(lo), None: last)
let found_by = V.group(by: v => earliest[v])
let later = earliest.suffix_min(None: last)

var best = SegTree(min, [V.cuts] c => if c == V.cuts.last!: 0 else: inf, act: add)
for i in V.drop(1).rev:
    for v in found_by[Some(i)]:
        best.apply(v.r.., 1)
    if earliest[i] == None:
        best.apply(i.r.., 1)
    let next = V.cuts.first_true(c => later[c].is_none_or(e => e >= i))!
    best[i.l] = best.fold(max(next, i.r)..) 
print(best[V.first!.next!.l])

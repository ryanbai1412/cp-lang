// 2042D Recommendations: predictors of i are j with l_j <= l_i and r_j >= r_i.
// The tightest right end comes from a sweep in (l asc, r desc) order with a
// sorted set of right ends; the left end is the same sweep on mirrored segments.
fn nearest_cover(segs: List<(l: Int, r: Int)>) -> List<?Int>:
    let order = segs.keys.sorted(by: i => (segs[i].l, desc(segs[i].r)))
    var rights = SortedSet<Int>.new()
    var out: List<?Int> = List.filled(segs.len, None)
    for i in order:
        out[i] = rights.first_true(x => x >= segs[i].r)
        rights.add(segs[i].r)
    out

@!multitest
let n: 1..=2e5 = read()
let segs = read<[n](l: 1..=1e9, r: 1..=1e9)>()
let seen = segs.tally(by: s => (s.l, s.r))
let hi = nearest_cover([(l: Int(s.l), r: Int(s.r)) for s in segs])
let lo = nearest_cover([(l: -Int(s.r), r: -Int(s.l)) for s in segs])
let answer = [if seen[(s.l, s.r)] > 1 or hi[i] == None: 0 else: (hi[i]! - s.r) + (s.l + lo[i]!) for (i, s) in segs.items]
print(answer, sep: "\n")

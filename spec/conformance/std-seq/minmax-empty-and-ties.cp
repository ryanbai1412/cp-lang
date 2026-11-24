// expect: ok, output "1 5 -1 2 b a\n0 0"
// `minmax` is `None` on an empty sequence, and among ties the min is the first and the max is the first; `unzip` of nothing is two empty lists.
let e: List<Int> = []
let (lo, hi) = [5, 1, 4, 5, 1].minmax()!
print(lo, hi, (e.minmax() ?? (-1, -1)).0, ([(2, 'a'), (1, 'b'), (2, 'c')].minmax(by: p => p.0)!.1).0, ([(2, 'a'), (1, 'b'), (2, 'c')].minmax(by: p => p.0)!.0).1, ([(2, 'a'), (2, 'c')].minmax(by: p => p.0)!.1).1)
let (u, v) = e.map(x => (x, x)).collect().unzip()
print(u.len, v.len)

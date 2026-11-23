// Inversion count: n <= 2e5 values up to 1e9; count pairs i < j with a[i] > a[j].
// Compress the values, sweep left to right, add the earlier values greater than a[j].
domain I = 0..read<1..=2e5>()
let a = read<[I]1..=1e9>()
let (domain C, comp) = compress(a)
var seen = Fenwick<C, Int>()
var total = 0
for x in a:
    let c = comp.at(x)
    total += seen.fold(c.r..)
    seen.add(c, 1)
print(total)

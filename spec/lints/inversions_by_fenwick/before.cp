let n: 1..=200000 = read()
domain I = 0..n
let a: [I]1..=1e9 = read()
let (domain C, comp) = compress(a)
var bit = Fenwick<C, Int>()
var inv = 0
for i in I:
    let id = comp.index_of(a[i])
    inv += i.ord - bit.prefix(id.r)
    bit.add(id, 1)
print(inv)

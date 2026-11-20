domain D = 0..10
var bs = Bitset<D>()
bs.add(D(3)); bs.toggle(D(4)); bs.remove(D(3))
print(D(4) in bs, bs.len, bs.shifted(2).len, (bs | bs).len)

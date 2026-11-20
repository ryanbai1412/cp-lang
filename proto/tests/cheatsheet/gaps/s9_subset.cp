domain D = 0..10
var sb = Subset<D>()
sb = sb.with(D(1)).with(D(2)).without(D(1))
print(sb.len, sum(1 for g in sb.subsets), sb.shifted(-3).len)

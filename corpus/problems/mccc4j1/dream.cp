// mccc4j1: three prices and three counts: print the total count and the total cost.
let (price, count) = read<([3]Int, [3]Int)>()
print(count.sum(), zip(price, count).map(((p, c)) => p * c).sum())

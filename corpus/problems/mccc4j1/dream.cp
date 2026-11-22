// MCCC 4 J1: three item prices and three counts; print the total count and the total cost.
let (price, count) = read ([3]Int, [3]Int)
print(count.sum(), zip(price, count).sum(of: (p, c) => p * c))

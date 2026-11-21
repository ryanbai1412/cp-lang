type Z = Mod<998244353>

let (n, seed): (Int, Int) = read()
domain D = 0..n
var dp = Prefix<D, Z>.filling(order: FillOrder.Forward)
var s = seed
for i in D:
    s = (s * 1103515245 + 12345) & 0x7fffffff
    let val = if i.ord == 0: Z(1) else:
        let lo = s & (i.ord - 1)
        dp.sum(D(lo)..i) + Z(1)
    dp.set(i, val)
print(dp.sum(D.first..=D.last))

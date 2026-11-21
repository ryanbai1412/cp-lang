type Z = Mod<998244353>

let (n, seed): (Int, Int) = read()
domain D = 0..n
var dp = Prefix<D, Z>.filling(order: FillOrder.Reverse)
var s = seed
for i in D.rev:
    s = (s * 1103515245 + 12345) & 0x7fffffff
    let rem = n - 1 - i.ord
    let val = if rem == 0: Z(1) else:
        let hi = i.ord + 1 + (s & (rem - 1))
        dp.sum(D(i.ord + 1)..=D(hi)) + Z(1)
    dp.set(i, val)
print(dp.sum(D.first..=D.last))

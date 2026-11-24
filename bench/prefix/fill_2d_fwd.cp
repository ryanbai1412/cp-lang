type Z = Mod<998244353>

let (h, w, seed): (Int, Int, Int) = read()
domain R = 0..h
domain C = 0..w
var dp = Prefix<R, C, Z>.filling(order: FillOrder.Forward)
var s = seed
for r in R:
    for c in C:
        s = (s * 1103515245 + 12345) & 0x7fffffff
        let val = if r.ord == 0 or c.ord == 0: Z(1) else:
            let r0 = s & (r.ord - 1)
            s = (s * 1103515245 + 12345) & 0x7fffffff
            let c0 = s & (c.ord - 1)
            dp.fold(R(r0)..r, C(c0)..c) + Z(1)
        dp.set(r, c, val)
print(dp.fold(R.first!..=R.last!, C.first!..=C.last!))

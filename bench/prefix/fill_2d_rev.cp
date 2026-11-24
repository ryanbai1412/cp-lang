type Z = Mod<998244353>

let (h, w, seed): (Int, Int, Int) = read()
domain R = 0..h
domain C = 0..w
var dp = Prefix<R, C, Z>.filling(order: FillOrder.Reverse)
var s = seed
for r in R.rev:
    for c in C.rev:
        s = (s * 1103515245 + 12345) & 0x7fffffff
        let rr = h - 1 - r.ord
        let rc = w - 1 - c.ord
        let val = if rr == 0 or rc == 0: Z(1) else:
            let r1 = r.ord + 1 + (s & (rr - 1))
            s = (s * 1103515245 + 12345) & 0x7fffffff
            let c1 = c.ord + 1 + (s & (rc - 1))
            dp.fold(R(r.ord + 1)..=R(r1), C(c.ord + 1)..=C(c1)) + Z(1)
        dp.set(r, c, val)
print(dp.fold(R.first!..=R.last!, C.first!..=C.last!))

type Z = Mod<1_000_000_007>
let n: 0..=1e18 = read()
let step = Mat<Z>.of([[Z(1), Z(1)], [Z(1), Z(0)]])
print((step ** n).get(0, 1))

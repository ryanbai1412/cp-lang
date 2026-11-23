type Z = Mod<1000000007>
let (a, b): (0..=1000000006, 0..=1000000006) = read()
let p = Z(a) * Z(b)
let q = p * p
print(p, q)

// `Prefix` over a grid needs its three type arguments spelled out.
domain R = 0..2
domain C = 0..3
let p = Prefix([R, C] (r, c) => 1 + 3 * Int(r) + Int(c))
print(p.fold(0..=1, 0..=2))

// expect: E0301
// Acceptance for seq/U70: a position found in `a` keeps `a`'s index space, so it can't index `b`.
domain I = 0..3
domain J = 0..3
let a: [I]Int = [1, 2, 3]
let b: [J]Int = [4, 5, 6]
let i = a.keys.find(i => a[i] > 1)!
print(b[i]) // <- here

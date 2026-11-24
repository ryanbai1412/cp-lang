// expect: E0302
// rule: 07.1.6
// A scan over `[I]T` is keyed by `I`'s positions and cuts; a position of another domain is not one of them.
domain I = 0..3
domain J = 0..3
let b: [I]Int = [5, -3, -2]
let pre = b.scan(+)
print(pre[J(1)]) // <- here

// Codeforces 2261 B: an adjacent pair that differs first at bit d can raise the
// larger side to all ones from bit d down; take the best such value.
@!multitest
let n: 1..=2e5 = read()
let a: [0..n]Int = read()
print(max(x | ((1 << bit_width(x ^ y)) - 1) for (x, y) in a.adjacent if x != y) ?? a[0])

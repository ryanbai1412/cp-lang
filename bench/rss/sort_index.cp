let n: 1..=2_000_000 = read()
domain I = 0..n
let a: [I]0..=1_000_000_000 = read()
var ord = I.collect()
ord.sort(by: i => (a[i], i))
print(ord[0].ord, ord[n - 1].ord)

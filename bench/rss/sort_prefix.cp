// 1e6 positions sorted by a prefix sum and then position, as cf2222d does.
let n: 1..=1_000_000 = read()
domain I = 0..n
let a: [I]0..=1_000_000_000 = read()
let pref = prefix(a)
var ord = I.collect()
ord.sort(by: i => (pref.prefix(i.l), i))
print(ord[0].ord, ord[n - 1].ord)

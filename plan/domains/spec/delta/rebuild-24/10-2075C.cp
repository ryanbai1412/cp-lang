// CF 2075C Two Colors. fits[v] counts paints that cover at least v planks; a split with
// i planks left and n - i right takes an ordered pair of distinct paints.
@!multitest
let n, m: 2..=2e5 = read()
let a = read<[m]1..=n>()
var fits: [1..=n]Int = 0
for x in a: fits[x] += 1
for v in (1..n).rev: fits[v] += fits[v + 1]
print((1..n).map(i => fits[i] * fits[n - i] - fits[max(i, n - i)]).sum())

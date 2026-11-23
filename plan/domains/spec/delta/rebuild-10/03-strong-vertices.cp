// CF 1857D Strong Vertices
// in: t; n, a_1..a_n, b_1..b_n | out: per test the count of argmax(a_i - b_i), then those 1-based indices ascending
@!multitest
domain V = 1..=read<2..=2e5>()
let a = read<[V]-1e9..=1e9>()
let b = read<[V]-1e9..=1e9>()

let gap = [V] v => a[v] - b[v]
let top = gap.max()!
let strong = [v for v in V if gap[v] == top]
print(strong.len)
print(strong)

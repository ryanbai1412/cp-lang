// Domains: positions print as their labels, bounds sit between
// positions, and arrays are indexed by positions only.
let n: 1..=20 = read()
domain V = 1..=n
let a: [V]Int = read()
var pre: [V.bounds]Int = 0
for v in V:
    pre[v.r] = pre[v.l] + a[v]
print(pre)
print(V.len, V.first, V.last)
var best = V.first
for v in V:
    if a[v] > a[best]:
        best = v
print("largest at", best, "=", a[best])
for v in V.rev:
    print(v, a[v], v.to_idx, v.rank)
print(sum(a[v] for v in V), sum(a), max(a), min(a))
print(count(a[v] > 0 for v in V), any(a[v] == 4 for v in V), all(a[v] > -100 for v in V))
let m: 1..=20 = read()
domain R = 0..m
var grid: [R, V]Int = 0
for r in R:
    for v in V:
        grid[r, v] = r.ord * 10 + a[v]
print(grid)
var flags: [V]Bool = false
for v in V:
    flags[v] = a[v] % 2 == 0
print(flags)
print(count(flags[v] for v in V))

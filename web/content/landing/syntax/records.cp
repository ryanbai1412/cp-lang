let (n, m) = read<(1..=2e5, 1..=2e5)>()
domain V = 1..=n
domain Road = [m]
let roads = read<[Road](u: V, v: V, r: 0..=1e9, p: 0..=1e9)>()
var ans: [V]?Int = None
for road in roads:
    ans[road.u] = max(ans[road.u] ?? 0, road.r - road.p)
print(ans, none: -1)

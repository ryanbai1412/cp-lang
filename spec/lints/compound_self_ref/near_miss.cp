var acc = 0
var best = 0
for d in [3, 1, 4]:
    acc = acc * 10 + d
    best = d - best
print(acc, best)
var low: ?Int = Some(2)
for d in [3, 1, 4]:
    low = low + d
print(low ?? 0)
var g = [[Some(1), Some(2)], [None, Some(4)]]
g[0][1] = g[0][1] + 5
print(g[0][1] ?? 0)

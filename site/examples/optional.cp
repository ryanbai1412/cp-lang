let n: 1..=100 = read()
domain V = 1..=n
let a: [V]Int = read()

var best: ?Int = none
for v in V:
    if a[v] % 2 == 0:
        best = max(best ?? a[v], a[v])
print(best ?? -1)

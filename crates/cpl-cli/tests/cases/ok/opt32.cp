// cp: 0.1 4cb76d1+dirty
let n: 1..=20 = read()
domain V = 1..=n
var best: [V]?0..=1000000 = none
for v in V:
    if v.idx % 3 != 0:
        best[v] = v.idx * 1000
var total = 0
for v in V:
    if let b = best[v]:
        total += b
print(total, count(best[v] == none for v in V))

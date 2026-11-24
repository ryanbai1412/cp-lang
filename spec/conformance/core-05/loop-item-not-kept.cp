// expect: E0603
// rule: 05.7.4 05.9.2
let rows = [[1, 2, 3], [4]]
var best = [0]
for r in rows:
    if r.len > best.len: best = r // <- here
print(best.len)

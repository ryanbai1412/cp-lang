// expect: ok, output "3"
// rule: 05.9.2
let rows = [[1, 2, 3], [4]]
var best = [0]
for r in rows:
    if r.len > best.len: best = r.clone()
print(best.len)

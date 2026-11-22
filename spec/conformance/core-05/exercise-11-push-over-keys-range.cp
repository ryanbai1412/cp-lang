// expect: ok
// rule: 05.9.1
var rows = List(List(1, 2), List(3))
let i = 0
for k in rows[i].keys: rows[i].push(rows[i][k])

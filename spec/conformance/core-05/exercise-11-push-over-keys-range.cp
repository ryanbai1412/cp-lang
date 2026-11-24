// expect: ok
// rule: 05.9.1
var rows = [[1, 2], [3]]
let i = 0
for k in rows[i].keys: rows[i].push(rows[i][k])

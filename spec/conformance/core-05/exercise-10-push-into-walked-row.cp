// expect: E0602 at 3
// rule: 05.9.1
var rows = List(List(1, 2), List(3))
let i = 0
for x in rows[i]: rows[i].push(x)

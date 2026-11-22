// expect: E0602 at 4
// rule: 05.7.6
var rows = List(List(1, 2), List(3))
let i = 0
let row = rows[i]
rows.push(List())
print(row.len)

// expect: E0602 at 3
// rule: 05.3.5 05.8.4
var rows = [[1], [2]]
let row = rows[0]
rows.push(List())
print(row.len)

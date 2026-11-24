// expect: E0602
// rule: 05.3.5 05.6.4
var rows = [[1, 2], [3]]
let row = rows[0]
rows.push([4]) // <- here
print(row.len)

// expect: E0602
// rule: 05.7.6
var rows = [[1, 2], [3]]
let i = 0
let row = rows[i]
rows.push(List()) // <- here
print(row.len)

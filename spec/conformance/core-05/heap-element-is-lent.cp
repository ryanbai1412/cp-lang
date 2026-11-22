// expect: E0602 at 3
// rule: 05.3.5 05.6.4
var rows = [[1, 2], [3]]
let row = rows[0]
rows.push([4])
print(row.len)

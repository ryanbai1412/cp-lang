// expect: ok, output "1 2"
// rule: 05.8.4
var rows = [[1]]
let n = rows[0].len
rows.push(List())
print(n, rows.len)

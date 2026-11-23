// expect: ok, output "2 2"
// rule: 05.8.2
// Binding `rows[0]` lends only that element; `rows[1]` stays writable.
var rows = [[1, 2], [3]]
let r = rows[0]
rows[1].push(5)
print(r.len, rows[1].len)

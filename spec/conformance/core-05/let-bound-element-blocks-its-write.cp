// expect: E0602
// rule: 05.8.2
// While `r` lends `rows[0]`, writing `rows[0]` is rejected.
var rows = [[1, 2], [3]]
let r = rows[0]
rows[0].push(5) // <- here
print(r.len)

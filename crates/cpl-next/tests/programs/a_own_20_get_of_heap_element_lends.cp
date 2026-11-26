// expect: E0602
// rule: R05.5.3
// `get` returns a cell of its receiver, so `row` borrows `rows`: pushing to `rows` while `row` is used is an error.
var rows = [[1], [2]]
let row = rows.get(0)
rows.push([3])  // <- here
print(row.len)

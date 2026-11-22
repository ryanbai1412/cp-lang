// expect: ok, output "1 9 2"
// rule: 02.8.4
var xs = [1, 2]
var y = xs[0]
y = 9
let rows = [[1, 2], [3]]
let row = rows[0]
print(xs[0], y, row.len)

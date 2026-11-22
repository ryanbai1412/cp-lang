// expect: E0602 at 3
// rule: 05.3.5
var g = [[1, 2], [3]]
let row = g[0]
g.push([4])
print(row[0])

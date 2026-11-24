// expect: E0603 at 5
// rule: 05.9.2
var rows = [[1, 2], [3]]
let i = 0
var longest = List<Int>()
for row in rows:
    if row.len > longest.len: longest = row

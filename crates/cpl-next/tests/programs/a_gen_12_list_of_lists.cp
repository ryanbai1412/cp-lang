// expect: ok, output "0 1 2"
// rule: R05.2
// List of lists: inner lists are values; pushing to a copy does not change the original (R05.2.2 deep copy on clone / move).
var grid: List<List<Int>> = List.new()
for i in 0..3:
    var row: List<Int> = List.new()
    for j in 0..i: row.push(j)
    grid.push(row)
print(grid[0].len, grid[1].len, grid[2].len)

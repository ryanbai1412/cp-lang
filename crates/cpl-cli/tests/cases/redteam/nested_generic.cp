var g: List<List<Int>> = List<List<Int>>()
g.push([1, 2, 3])
g.push(List<Int>())
g.push([4])
print(g.len, g[0].len, g[1].len, g[2][0])
var total = 0
for row in g:
    for x in row: total += x
print(total)

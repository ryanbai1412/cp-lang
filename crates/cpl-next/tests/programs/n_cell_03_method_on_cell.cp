// expect: ok, output "2 5 3\n3 2\n1 4"
// rule: R12.4.6
var g = [[1], [2, 3]]
g[0].push(5)
print(g[0].len, g[0][1], g[1][1])
g[1].push(g[0][0] + 1)
print(g[1].len, g[1][2])
var h = [[[1]], [[2]]]
h[1][0].push(4)
print(h[0][0][0], h[1][0][1])

// expect: ok, output "1 9 3\n10\n7 5\n4\nxbc\n0 0 1"
// rule: R12.4.6
var xs = [1, 2, 3]
xs[1] = 9
print(xs[0], xs[1], xs[2])
xs[1] += 1
print(xs[1])
var g = [[1], [2]]
g[1][0] = 7
g[0][0] += 4
print(g[1][0], g[0][0])
domain V = 0..3
var a: [V]Int = 0
a[V(2)] = 4
print(a[V(2)])
var s = "abc"
s[0] = 'x'
print(s)
var p = [0, 0, 1]
p[2] = p[p[2]]
print(p[0], p[1], p[2] + 1)

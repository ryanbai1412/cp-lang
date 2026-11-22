// expect: ok, output "0 0"
// rule: std.dsu.Dsu.roots
domain E = 0..0
var d = Dsu<E>()
print(d.components, d.roots().collect().len)

// expect: ok, output "2\n0 1 2 3\n2"
// rule: 04.2.22
// Every domain position has `.ord`, its 0-based ordinal; an enum's variants are positions, so `Dir.S.ord` is 2, read directly or from a tuple field.
enum Dir = N | E | S | W
print(Dir.S.ord)
print([d.ord for d in Dir])
let p = (1, Dir.S)
print(p.1.ord)

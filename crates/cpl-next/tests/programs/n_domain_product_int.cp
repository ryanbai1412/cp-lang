// expect: ok, output "1 5 7\n23"
// rule: R04.2.15 R04.2.9
// `Int(p)` on a product position is its row-major ordinal, for two, three and four factors.
domain R = 0..2
domain C = 0..3
domain D = (R, C)
type Cube = (R, C, R)
print(Int(D((0, 1))), Int(D((1, 2))), Int(Cube((1, 0, 1))))
var last = 0
for p in (R, C, R, R): last = Int(p)
print(last)

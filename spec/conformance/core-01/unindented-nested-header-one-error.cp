// expect: E0104
// rule: 01.2.3
// lacks: "E0201"
// An unindented header under an unindented loop body stays in the loop: one E0104, and the body still sees `x`.
let a = [5, 20]
for x in a:
if x > 10: // <- here
print(x)

// expect: ok, output "1 2\n3 x\nSouth"
// rule: 10.3.5
struct P: a: Int, b: Int
enum Dir = North | South
print(P(1, 2))
print((3, "x"))
print(South)

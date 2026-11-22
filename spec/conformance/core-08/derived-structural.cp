// expect: ok, output "3 YES YES"
// rule: 08.1.6
struct P: a: Int, b: Int
enum Lvl = Lo | Hi
let xs = [P(2, 1), P(1, 5), P(1, 3)].sorted()
print(xs[0].b, yes(P(1, 2) == P(1, 2)), yes(Lo < Hi))

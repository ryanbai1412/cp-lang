// expect: ok, output "1 2"
// rule: 01.7.1
// An `if let` clause in a comprehension binds for the clauses after it.
enum Cmd = Set(a: Int, b: Int) | Get(a: Int)
let cmds = [Set(1, 2), Get(3)]
print([x for c in cmds if let Set(a, b) = c for x in [a, b]])

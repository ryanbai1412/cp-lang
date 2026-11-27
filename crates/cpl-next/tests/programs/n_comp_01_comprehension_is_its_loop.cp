// expect: ok, output "1 4 9\n2 3\n0 0 1 1 2 2\n1\n5 6"
// A comprehension is the loop its clauses write, pushing its element where every condition holds;
// clauses nest outermost first, and an `if` takes `let` binders and commas (R01.8.1).
let xs = [1, 2, 3, 4]
print([x * x for x in xs if x < 4])
print([x for x in xs if x > 1, x < 4])
let ps = [(i, j) for i in 0..3, j in i + 1..3]
print([p.0 for p in ps], [p.1 for p in ps])
let os = [Some(3), None, Some(-1)]
print([y for o in os if let y = o, y > 0].len * 3 / 3)
print([[x + 3 for x in xs if x > 1][k] for k in 0..2])

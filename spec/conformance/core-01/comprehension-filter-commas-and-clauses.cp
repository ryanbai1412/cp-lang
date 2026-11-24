// expect: ok, output "2 3\n3 1\n4 1\n5"
// rule: 01.8.1
// A comma after a comprehension's `if` joins another condition, unless it starts the next `x in xs` clause.
let xs = [1, 2, 3, 4]
print([x for x in xs if x > 1, x < 4])
print([(x, y) for x in xs if x > 2, y in xs if y < 2])
let ys: List<?Int> = [Some(1), None, Some(5)]
print([y for x in ys if let y = x if y > 1])

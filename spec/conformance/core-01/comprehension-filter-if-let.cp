// expect: ok, output "[3]"
// rule: 01.8.1
// A comprehension's `if` takes what an `if` header takes, `let` binders and commas included (rebuild round 30 F04).
let xs: List<?Int> = [Some(1), None, Some(3)]
print([y for x in xs if let y = x, y > 1])

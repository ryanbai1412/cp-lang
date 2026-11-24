// expect: E0204
// rule: 06.1.2
// has: "needs a type"
fn around(pre, w: Int) -> Int = w // <- here
print(around(1, 2))

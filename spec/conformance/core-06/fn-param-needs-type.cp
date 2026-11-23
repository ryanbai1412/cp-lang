// expect: E0204 at 1
// rule: 06.1.2
// has: "needs a type"
fn around(pre, w: Int) -> Int = w
print(around(1, 2))

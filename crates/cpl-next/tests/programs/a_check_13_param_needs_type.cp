// expect: E0204
// rule: 06.1.2
// A `fn` parameter always has a type; only a lambda's take theirs from where it is passed.
fn around(pre, w: Int) -> Int = w  // <- here
print(around(1, 2))

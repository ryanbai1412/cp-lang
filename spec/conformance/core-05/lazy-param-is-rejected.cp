// expect: E0503
// rule: 05.2.3
// has: "lazy"
// A parameter is lent, `mut` or `var`; a deferred value is a `() -> T`.
fn pick(c: Bool, lazy x: Int) -> Int: // <- here
    if c: x else: 0
print(pick(true, 1))

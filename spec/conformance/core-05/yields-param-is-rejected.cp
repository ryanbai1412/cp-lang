// expect: E0503
// rule: 05.2.3
// has: "yields"
// A parameter is lent, `mut` or `var`; only a fn's result yields.
fn total(xs: yields Int) -> Int: // <- here
    0
print(0)

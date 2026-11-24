// expect: E0505
// rule: 06.4.9
// tool: check
// has: "same `from` twice"
@lint(by_hand, from: "x * x", to: "sq(x)")
pub fn sq(x: Int) -> Int:
    x * x
@lint(by_hand, from: "x * x", to: "square(x)") // <- here
pub fn square(x: Int) -> Int:
    x * x

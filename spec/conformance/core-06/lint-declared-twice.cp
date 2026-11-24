// expect: E0505
// rule: 06.4.9
// tool: check
// has: "declared twice"
@lint(by_hand, from: "x * x", to: "sq(x)")
pub fn sq(x: Int) -> Int:
    x * x
@lint(by_hand, from: "x * x * x", to: "cube(x)") // <- here
pub fn cube(x: Int) -> Int:
    x * x * x

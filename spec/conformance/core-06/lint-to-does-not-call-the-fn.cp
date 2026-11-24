// expect: E0505
// rule: 06.4.9
// tool: check
// has: "does not call `sq`"
@lint(square_by_hand, from: "x * x", to: "x ** 2") // <- here
pub fn sq(x: Int) -> Int:
    x * x

// expect: E0505 at 1
// rule: 06.4.9
// tool: check
// has: "does not call `sq`"
@lint(square_by_hand, from: "x * x", to: "x ** 2")
pub fn sq(x: Int) -> Int:
    x * x

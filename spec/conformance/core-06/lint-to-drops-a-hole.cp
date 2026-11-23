// expect: E0505 at 1
// rule: 06.4.9
// tool: check
// has: "drops `y`"
@lint(mul_by_hand, from: "x * y", to: "mul(x, x)")
pub fn mul(x: Int, y: Int) -> Int:
    x * y

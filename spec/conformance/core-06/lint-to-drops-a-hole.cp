// expect: E0505
// rule: 06.4.9
// tool: check
// has: "drops `y`"
@lint(mul_by_hand, from: "x * y", to: "mul(x, x)") // <- here
pub fn mul(x: Int, y: Int) -> Int:
    x * y

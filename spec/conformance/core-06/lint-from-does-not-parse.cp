// expect: E0505
// rule: 06.4.9
// tool: check
// has: "is not one expression or statement"
@lint(square_by_hand, from: "x * * x", to: "sq(x)") // <- here
pub fn sq(x: Int) -> Int:
    x * x

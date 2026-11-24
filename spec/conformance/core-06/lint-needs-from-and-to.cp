// expect: E0505
// rule: 06.4.9
// tool: check
// has: "takes a name, `from:` and `to:`"
@lint(square_by_hand, from: "x * x") // <- here
pub fn sq(x: Int) -> Int:
    x * x

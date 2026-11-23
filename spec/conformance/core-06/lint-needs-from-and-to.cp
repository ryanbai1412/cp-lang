// expect: E0505 at 1
// rule: 06.4.9
// tool: check
// has: "takes a name, `from:` and `to:`"
@lint(square_by_hand, from: "x * x")
pub fn sq(x: Int) -> Int:
    x * x

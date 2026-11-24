// expect: E0505
// rule: 06.4.9
// tool: check
// has: "`when:` of `@lint(up_by_hand)` is not holes `from` uses"
@lint(up_by_hand, from: "(a + b - 1) / b", to: "up(a, b)", when: "b != 0") // <- here
pub fn up(a: Int, b: Int) -> Int:
    a

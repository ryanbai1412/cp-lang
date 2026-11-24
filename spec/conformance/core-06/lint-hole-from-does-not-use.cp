// expect: E0505
// rule: 06.4.9
// tool: check
// has: "names `n`, which `from` does not use"
@lint(first_by_hand, from: "xs[0]", to: "first(xs)", holes: "n") // <- here
pub fn first(xs: List<Int>) -> Int:
    xs[0]

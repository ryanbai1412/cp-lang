// expect: E0505
// rule: 06.4.9
// tool: check
// has: "names `n`, which neither `from` nor `also:` uses"
@lint(first_by_hand, from: "xs[0]", to: "first(xs)", holes: "n") // <- here
pub fn first(xs: List<Int>) -> Int:
    xs[0]

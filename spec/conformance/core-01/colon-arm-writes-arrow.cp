// expect: E0503
// rule: 01.7.3
// has: "=>"
// A `match` arm is `pattern => body`; `pattern:` is an error whose fix writes `=>` (decision 4.1).
let k = 2
match k:
    0: print("zero") // <- here
    _ => print("other")

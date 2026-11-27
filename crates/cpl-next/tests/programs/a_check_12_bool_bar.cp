// expect: E0309
// rule: 01.5.9
// `|` is an integer operator; on `Bool`s the error names `or`.
let p = true
let q = 3 > 2
print(yes(q | p))  // <- here

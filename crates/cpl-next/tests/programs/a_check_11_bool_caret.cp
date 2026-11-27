// expect: E0309
// rule: 01.5.9
// `^` is an integer operator; on `Bool`s the error names `!=`.
let p = true
print(yes(p ^ false))  // <- here

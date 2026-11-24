// expect: E0201
// rule: 07.4.16
// std has no `suffix_fold`: a suffix fold is `a.rev.scan(op)`, its one spelling (decision 511).
let a = [2, 3, 4]
print(suffix_fold(+, a)[0]) // <- here

// expect: E0201
// rule: 07.4.16
// has: "a.rev.scan(+)"
// std has no `suffix_fold`: the error names the scan of the reversed view, and its fix writes it (decision 511).
let a = [2, 3, 4]
let s = suffix_fold(+, a) // <- here

// expect: E0302 at 2
// rule: 07.2.11
let a = [1, 2, 3]
let xs: List<Int> = a.rev
print(xs.len)

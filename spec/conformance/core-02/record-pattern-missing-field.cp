// expect: E0302 at 2
// rule: 02.5.5
// A record pattern that leaves a field out without `..` is an error.
let e = (u: 1, v: 2, w: 9)
let (u:, v:) = e
print(u + v)

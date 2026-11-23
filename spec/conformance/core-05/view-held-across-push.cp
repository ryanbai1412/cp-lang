// expect: E0602 at 6
// rule: 05.7.6
// A push may reallocate the list, so it is rejected while a named view of the list is used below it.
var a = [1, 2, 3]
let v = a[0..2]
a.push(4)
print(v.len)

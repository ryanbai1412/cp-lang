// expect: E0602 at 4
// rule: 07.2.6 05.7.5
// A view borrows its source (R07.2.6): writing the source while the view is still used later is an overlap.
var a = [1, 2, 3]
let s = a[0..2]
a[0] = 9
print(s.sum())

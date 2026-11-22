// expect: E0602 at 3
// rule: 05.3.4 05.7.7
var a = [1, 2, 3]
let p = mut a[1]
print(a[0])
p += 1

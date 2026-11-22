// expect: E0602 at 3
// rule: 07.2.16
var a = [1, 2, 3]
let v = a.rev
a.push(4)
print(v.len)

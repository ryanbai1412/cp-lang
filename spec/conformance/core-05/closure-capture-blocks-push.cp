// expect: E0602 at 3
// rule: 05.7.5
var a = [1]
let f = () => a.len
a.push(2)
print(f())

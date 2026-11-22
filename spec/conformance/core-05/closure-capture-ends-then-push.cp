// expect: ok, output "1\n2"
// rule: 05.7.5
var a = [1]
let f = () => a.len
print(f())
a.push(2)
print(a.len)

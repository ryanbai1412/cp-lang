// expect: ok, output "1 0"
// rule: 01.6.4
var d: ?Int = None
let a = if d min= 5: 1 else: 0
let b = if d min= 7: 1 else: 0
print(a, b)

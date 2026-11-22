// expect: ok, output "3"
// rule: 02.10.1
var x: ?Int = None
x = Some(3)
print(x ?? 0)

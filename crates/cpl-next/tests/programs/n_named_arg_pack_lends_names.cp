// expect: ok, output "3,1,2\n3\n2 1 3|ab\n3 3"
// rule: 05.4.4
// A variadic pack's names are read where the pack is made, also when a named argument comes
// first or a part is a call, so `print(xs, sep: ",")` lends `xs` and does not move it.
let xs = [3, 1, 2]
print(xs, sep: ",")
print(xs.len)
var s = "a"
s += "b"
print(sep: " ", xs.rev, end: "|")
print(s)
print(xs.len, s.len + 1, sep: " ")

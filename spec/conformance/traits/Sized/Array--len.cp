// expect: ok, output "3"
// engines: next
domain V = 0..3
var a = [V] v => Int(v) * 10
print(a.len)

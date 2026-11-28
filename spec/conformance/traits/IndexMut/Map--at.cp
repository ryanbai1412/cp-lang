// expect: ok, output "15 3 2"
// engines: next
var m = [1: 10]
m[1] += 5
m[2] += 3
print(m[1], m[2], m.len)

// expect: ok, output "2 3 3"
// engines: next
let a = ([1, 2], 3)
var b = a.clone()
b._0.push(9)
print(a._0.len, b._0.len, b._1)

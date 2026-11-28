// expect: ok, output "1 2 x 4"
// engines: next
let a = ([1], "x", 4)
var b = a.clone()
b._0.push(2)
print(a._0.len, b._0.len, b._1, b._2)

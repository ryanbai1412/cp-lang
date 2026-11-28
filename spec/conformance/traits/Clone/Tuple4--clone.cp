// expect: ok, output "1 2 1 c t"
// engines: next
let a = (1, [2], 'c', true)
var b = a.clone()
b._1.push(3)
print(a._1.len, b._1.len, b._0, b._2, if b._3: "t" else: "f")

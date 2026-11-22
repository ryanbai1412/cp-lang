// expect: ok, output "0 1 0"
// rule: 07.12.8
var ids = Map<Str, Int>()
let x = ids.get_or_insert("a", ids.len)
let y = ids.get_or_insert("b", ids.len)
let z = ids.get_or_insert("a", ids.len)
print(x, y, z)

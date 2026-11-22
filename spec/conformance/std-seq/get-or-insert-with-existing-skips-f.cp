// expect: ok, output "3 1"
var m: Map<Int, Int> = Map()
m[1] = 3
let v = m.get_or_insert_with(1, () => 5)
print(v, m.len)

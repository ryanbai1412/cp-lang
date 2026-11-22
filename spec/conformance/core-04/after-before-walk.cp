// expect: ok, output "3 4 0 1"
// rule: 04.2.19
domain V = 0..5
let v = V(2)
var out = List<Int>()
for j in v.after: out.push(Int(j))
for j in v.before: out.push(Int(j))
print(out)

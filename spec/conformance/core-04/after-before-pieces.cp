// expect: ok, output "2 2"
// rule: 04.2.19
domain V = 0..5
let v = V(2)
print(v.after.len, v.before.len)

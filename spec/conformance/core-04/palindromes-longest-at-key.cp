// expect: ok, output "3"
// rule: 04b.5.6
let p = manacher("xaba"[1..])
print(p.longest_at(2))

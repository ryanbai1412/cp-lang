// expect: ok, output "1\n2\n3"
// engines: next
domain V = 1..=3
for v in V: print(Int(v))

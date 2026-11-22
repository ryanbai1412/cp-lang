// expect: ok, output "2\n4"
// rule: 04.2.13
domain V = 0..5
print(Int(V.wrap(7)))
print(Int(V.wrap(-1)))

// expect: ok, output "3 10"
// rule: 07.11.5
let (domain C, comp) = compress([30, 10, 30, 20])
print(C.size, comp[C.first!])

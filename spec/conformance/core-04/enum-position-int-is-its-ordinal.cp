// expect: ok, output "0 1 2 3"
// rule: 04.2.16
enum D4 = Up | Right | Down | Left
print([Int(d) for d in D4])

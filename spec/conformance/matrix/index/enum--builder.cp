// expect: ok, output "4"
enum D = N | E | S | W
let a: [D]Int = [D] x => 1
print(a.sum())

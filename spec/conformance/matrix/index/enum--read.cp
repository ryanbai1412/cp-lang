// expect: ok, output "4"
// input: "1 1 1 1"
enum D = N | E | S | W
let a: [D]Int = read()
print(a.sum())

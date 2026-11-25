// expect: ok, output "4\n3"
// input: "2\n7 2\n9 3\n"
@!multitest
let (a, b): (1..=1000000000, 1..=1000000000) = read()
print((a + b - 1) / b)  // expect: ceil_div_formula

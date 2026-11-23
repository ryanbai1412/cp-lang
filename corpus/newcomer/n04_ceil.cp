@!multitest
let (a, b): (1..=1000000000, 1..=1000000000) = read()
print((a + b - 1) / b)  // expect: ceil_div_formula

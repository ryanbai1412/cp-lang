@!multitest
let (a, b): (1..=100, 1..=100) = read()
print(if a % b == 0: "YES" else: "NO")  // expect: bool_ternary_print

// expect: ok, output "NO"
// rule: 09.4.4
with lint(float_equality: allow)
let x = 0.1 + 0.2
print(yes(x == 0.3))

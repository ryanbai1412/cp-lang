// expect: L0001
// rule: 09.4.4
with lint(not_a_real_lint: allow) // <- here
let x = 1
print(x)

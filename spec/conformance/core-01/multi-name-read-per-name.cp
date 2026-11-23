// expect: ok, output "3 7"
// rule: 01.6.1
// input: "3 7\n"
// Several names sharing a type and an initializer run the initializer once per name.
let n, m: 1..=100 = read()
print(n, m)

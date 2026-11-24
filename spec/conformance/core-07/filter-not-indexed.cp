// expect: E0305
// rule: 07.1.2 07.2.13 07.7.2 07.5.12
let a = [1, 2, 3]
print(a.filter(x => x > 1)[0]) // <- here

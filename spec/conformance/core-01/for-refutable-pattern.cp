// expect: E0106
// rule: 01.6.2
let xs = [(1, 2)]
for (a, 2) in xs: // <- here
    print(a)

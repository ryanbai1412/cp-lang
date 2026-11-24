// expect: E0201
// rule: 05.10.6
let g = () => h() // <- here
let h = () => 1
print(g())

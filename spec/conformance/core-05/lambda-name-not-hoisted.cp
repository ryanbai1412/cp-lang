// expect: E0201 at 1
// rule: 05.10.6
let g = () => h()
let h = () => 1
print(g())

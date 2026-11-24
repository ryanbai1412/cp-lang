// expect: E0302
// rule: 03.5.1
let a: Mod<7> = 3
let b: Mod<11> = a // <- here
print(b)

// expect: E0102
// rule: 06.2.4
mut fn bump(mut x: Int): x += 1 // <- here
var y = 1
bump(mut y)
print(y)

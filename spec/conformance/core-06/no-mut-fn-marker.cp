// expect: E0102 at 1
// rule: 06.2.4
mut fn bump(mut x: Int): x += 1
var y = 1
bump(mut y)
print(y)

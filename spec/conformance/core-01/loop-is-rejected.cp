// expect: E0503
// rule: 01.7.2
// has: "while true"
// There is no `loop`: the loop that exits in the middle is `while true:`.
var i = 0
loop: // <- here
    i += 1
    if i > 2: break
print(i)

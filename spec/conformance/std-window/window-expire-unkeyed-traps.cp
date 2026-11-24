// expect: R0401 (checked)
var w = Window(min)
w.push(3)
w.expire(before: 1) // <- here

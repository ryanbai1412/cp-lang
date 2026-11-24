// expect: R0401 (checked)
var w = Window(min)
w.push(3, at: 2)
w.push(1, at: 1) // <- here

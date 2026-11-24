// expect: R0401 (checked)
var w = Window(+)
w.pop(0) // <- here
print(w.len)

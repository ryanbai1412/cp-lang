// expect: R0401 at 4 (checked)
// `Window` has one `pop(x)`, the `Track` one; a checked build asserts `x` is the element leaving the front, so an off-by-one in the caller traps instead of folding the wrong window (decision 414).
var w = Window(+)
w.push(1)
w.pop(2)
print(w.len)

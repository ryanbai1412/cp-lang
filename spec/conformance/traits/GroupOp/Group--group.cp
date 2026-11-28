// expect: ok, output "-2 5"
// engines: next
let g = Group<Int>(0, (a, b) => a + b, a => -a)
print(g.group().inv(2), g.group().op(2, 3))

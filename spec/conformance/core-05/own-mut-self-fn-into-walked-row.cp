// expect: E0602
// rule: 05.9.1
// A program's own `mut self` fn changes its receiver exactly as a std one does.
fn spin(mut self: List<Int>):
    self.rotate_left(1)
var rows = [[1, 2], [3]]
let i = 0
for x in rows[i]: rows[i].spin() // <- here

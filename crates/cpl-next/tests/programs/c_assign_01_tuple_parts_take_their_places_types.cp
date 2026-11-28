// expect: ok, output "1 0\n1 1 -1"
// rule: R01.6.4
// Each place of a tuple assignment gives its part the type it is built as: `[None]` is a list of the place's optional.
var a: List<?Int> = [Some(1)]
var b = [1]
(a, b) = ([None], [0])
print(a.len, b[0])
struct L(var lines: List<?Int>, var left: List<Int>):
    fn clear(mut self):
        (lines, left) = ([None], [0])
var m = L([Some(3), None], [1, 2])
m.clear()
print(m.lines.len, m.left.len, m.lines[0] ?? -1)

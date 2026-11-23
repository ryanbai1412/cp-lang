fn drain(mut xs: List<Int>) -> yields Int:
    while xs.len > 0:
        yield xs.pop()!
var v = [1, 2, 3]
let g = drain(mut v)
for x in g: print(x)

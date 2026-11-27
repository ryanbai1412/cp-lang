// expect: R0101
// has: "call stack: Box.at (line 10) <- top level (line 13)"
// lacks: "Box_"
// A trap in a loop body inside a method names the method; std's range generator is left out.
struct Box:
    xs: List<Int>
    fn at(self, n: Int) -> Int:
        var s = 0
        for i in 0..n:
            s += xs[i] // <- here
        return s
let b = Box(xs: [1, 2])
print(b.at(3))

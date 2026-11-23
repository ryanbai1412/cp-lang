// expect: exit 3
// rule: 09.6.1
// tool: run
// has: "call stack: Box.at ("
// lacks: "Box_"
// A trap inside a generic method names the frame `Box.at`, not the lowered name.
struct Box<T>:
    xs: List<T>
    fn at(self, i: Int) -> T:
        return xs[i]
let b = Box(xs: [1, 2])
print(b.at(5))

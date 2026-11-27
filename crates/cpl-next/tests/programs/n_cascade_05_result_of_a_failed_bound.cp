// expect: E0606
// rule: diagnostics
// lacks: "slice:"
// lacks: "expected"
// A call whose bound fails solves nothing through that bound: the result it leaves unknown is the
// error node, so printing it or adding to it says nothing more.
struct Box<S>(s: S)
impl Seq<T> for Box<S: Seq<T>>:
    fn iter(self) -> yields T:
        for x in s: yield x
let b = Box(7)
let n = b.sum() // <- here
print(n, n + 1)

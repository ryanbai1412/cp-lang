// expect: E0302
// rule: 12.4
// An impl whose inline bound does not hold for the target's parameter gives it nothing.
struct Wrap<S>(s: S)
impl Seq<T> for Wrap<S: Seq<T>>:
    fn iter(self) -> yields T:
        for x in s: yield x
let w = Wrap(5)
print(w.sum()) // <- here

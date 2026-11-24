// A @grader fn's own domain stays a domain inside it: `List<C>` is still a
// `List<C>` to std's generic fns, and becomes ints only at the grader.
@grader
fn f(p: [C]1..=9) -> List<C>:
    var t: List<C> = []
    t.push(C.first!)
    t.push(C.last!)
    return t.reversed()

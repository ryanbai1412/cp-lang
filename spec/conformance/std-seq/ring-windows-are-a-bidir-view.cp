// expect: ok, output "2 0 | 3"
// rule: 07.4.5
// `ring.windows(k)` is a view, Bidir and Sized: `.last` through a Bidir bound is the window starting last.
fn lst<S: Bidir<T>, T>(s: S) -> ?T = s.last
let w = lst([1, 2, 3].ring.windows(2))!
print([i for i in w.range], "|", [1, 2, 3].ring.windows(2).len)

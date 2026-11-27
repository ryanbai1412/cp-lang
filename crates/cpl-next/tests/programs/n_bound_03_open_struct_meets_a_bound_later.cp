// expect: ok, output "3 2"
// rule: R08.1.9
// A value whose type is still open (`Window<_, Monoid<_>>`) passed at a bound takes the impl
// its head matches, and the parts still open stay open with their fn sets until a later use
// fixes them.
use std.window.{Window}
use std.seq.{Track}
fn feed<St: Track<Int>>(s: St) -> Int:
    var st = s
    st.push(2)
    st.push(1)
    st.pop(2)
    1
let w = Window(+)
print(feed(w) + 2, feed(Window(+)) + 1)

// A getter that gives another lent getter's result lends it too, through
// any chain: `c.view[i]` reads `xs` in place, copying nothing per call.
struct A:
    xs: List<Int>
    @get pub fn view(self) -> List<Int> = xs
struct B:
    a: A
    @get pub fn view(self) -> List<Int> = a.view
struct C:
    b: B
    @get pub fn view(self) -> List<Int> = b.view
let c = C(B(A([i * i for i in 0..5])))
var s = 0
for i in 0..5: s += c.view[i]
let kept = c.view
print(s, kept, c.b.view.len)

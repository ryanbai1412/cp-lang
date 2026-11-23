// A method draining one `self` field, with a closure reading a sibling
// field, after the drain removes something (both engines agree).
use std.bag.*
struct W:
    ks: SortedBag<Int>
    h: HashMap<Int, Int>
    fn go(mut self) -> List<Int> = ks.drain(after: 1, while: k => h.get(k)! < 50)
var h = HashMap<Int, Int>()
h[4] = 40
h[6] = 60
var w = W(SortedBag.of([1, 4, 6]), h)
print(w.go())

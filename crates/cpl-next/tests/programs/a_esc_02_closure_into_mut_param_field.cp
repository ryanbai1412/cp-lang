// expect: E0603
// rule: R05.10.2, R8
// R8 through a mut parameter: the callee stores a lent closure in the caller's struct.
struct Box:
    var f: () -> Int
fn arm(mut b: Box):
    var c = 41
    b.f = () => { c += 1; c }  // <- here
var b = Box(() => 0)
arm(mut b)
print(b.f())

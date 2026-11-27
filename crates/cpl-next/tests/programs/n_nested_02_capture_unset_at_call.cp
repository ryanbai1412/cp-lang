// expect: E0205
// rule: R06.1.7
// A nested fn called above its statement needs every capture set at the call, its siblings' too.
fn g() -> Int:
    let a = 1
    let r = h() // <- here
    let k = 5
    fn h() -> Int = a + i()
    fn i() -> Int = k
    r
print(g())

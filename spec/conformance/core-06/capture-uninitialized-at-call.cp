// expect: E0205 at 2
// rule: 06.1.7
fn g() -> Int:
    let r = h()
    let k = 5
    fn h() -> Int = k
    r

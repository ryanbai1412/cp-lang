// expect: E0205
// rule: 06.1.7
fn g() -> Int:
    let r = h() // <- here
    let k = 5
    fn h() -> Int = k
    r

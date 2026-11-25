// expect: E0608
// rule: R05 R2
// A move in one branch of an `if` then a use after the `if`.
fn go(k: Int) -> Int:
    let a = [1, 2, 3]
    var n = 0
    if k > 0:
        let b = a
        n = b.len
    n + a.len  // <- here
print(go(1))

// expect: E0608
// rule: R05 R2
// R2 (E0608): `let b = a` moves the list; a later use of `a` is an error, inside a lambda body as anywhere.
fn go() -> Int:
    let f = () =>
        let a = [1, 2, 3]
        let b = a
        a.len + b.len  // <- here
    f()
print(go())

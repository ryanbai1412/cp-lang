// expect: E0503
// rule: 01.6.2
// has: "guard let"
// `let p = e else: ...` binds or leaves; there is no `guard let`.
fn half(x: ?Int) -> Int:
    guard let y = x else: return 0 // <- here
    y / 2
print(half(Some(8)))

// expect: E0302
// rule: 02.4.7 05.9.3
// `?` is never control flow, so `v` is a `?Int` and the fn does not return early.
fn f(o: ?Int) -> Int:
    let v = o?
    return v // <- here
print(f(None))

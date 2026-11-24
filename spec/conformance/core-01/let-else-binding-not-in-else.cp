// expect: E0201
// rule: 01.6.2
fn f(x: ?Int) -> Int:
    let v = x else:
        return v // <- here
    v
print(f(None))

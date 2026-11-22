// expect: E0201 at 3
// rule: 01.6.2
fn f(x: ?Int) -> Int:
    let v = x else:
        return v
    v
print(f(None))

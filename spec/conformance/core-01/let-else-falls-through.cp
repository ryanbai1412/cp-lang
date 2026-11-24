// expect: E0112
// rule: 01.6.2
fn f(x: ?Int) -> Int:
    let v = x else: // <- here
        print("none")
    v
print(f(None))

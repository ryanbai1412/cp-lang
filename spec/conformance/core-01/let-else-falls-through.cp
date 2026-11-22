// expect: E0112 at 2
// rule: 01.6.2
fn f(x: ?Int) -> Int:
    let v = x else:
        print("none")
    v
print(f(None))

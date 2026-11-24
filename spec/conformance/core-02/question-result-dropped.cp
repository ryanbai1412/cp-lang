// expect: E0804
// rule: 02.4.7
fn f(x: ?Int) -> ?Int:
    x? // <- here
    return x
print(f(Some(1)) ?? 0)

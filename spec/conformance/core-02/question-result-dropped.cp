// expect: E0804 at 2
// rule: 02.4.7
fn f(x: ?Int) -> ?Int:
    x?
    return x
print(f(Some(1)) ?? 0)

// error: let-else needs a ?T for `3`
fn f(a: ?Int) -> Int:
    let (x, y) = (a, 3) else: return 0
    x + y
print(f(None))

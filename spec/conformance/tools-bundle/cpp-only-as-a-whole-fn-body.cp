// expect: E0501 at 3
// rule: bundle.5
fn f(a: Int) -> Int:
    let b = a + 1
    @cpp("return b;")
print(f(1))

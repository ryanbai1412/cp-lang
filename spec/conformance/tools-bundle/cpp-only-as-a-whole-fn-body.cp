// expect: E0501
// rule: bundle.5
fn f(a: Int) -> Int:
    let b = a + 1
    @cpp("return b;") // <- here
print(f(1))

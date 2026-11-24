// expect: E0606
// rule: 07.2.10
fn g() -> yields Int:
    yield 1
print(g()[0]) // <- here

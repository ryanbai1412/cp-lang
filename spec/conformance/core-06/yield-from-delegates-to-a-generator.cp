// expect: ok, output "1\n2\n3"
// rule: 06.3.6
// `yield from e` in a generator body yields each of `e`'s values in turn.
fn inner() -> yields Int:
    yield 1
    yield 2
fn outer() -> yields Int:
    yield from inner()
    yield 3
for x in outer():
    print(x)

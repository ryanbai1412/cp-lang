// expect: ok, output "1\n3\n4\n9"
// rule: 07.2.10
// A `-> yields T` fn is a generator whatever its body: `= match` with a `yield` in each arm.
fn f(k: Int) -> yields Int = match k:
    0: yield 1
    _:
        yield k
        yield k + 1
fn g(k: Int) -> List<Int>:
    fn sq() -> yields Int = match k:
        0: yield 0
        _: yield k * k
    [x for x in sq()]
for x in f(0): print(x)
for x in f(3): print(x)
print(g(3))

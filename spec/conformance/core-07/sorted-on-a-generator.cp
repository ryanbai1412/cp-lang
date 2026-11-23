// expect: ok, output "1 2 3"
// rule: 07.11.2
// `sorted()` works on a walk-only sequence such as a generator, and materializes (rebuild round 18 H13).
fn gen() -> yields Int:
    yield 3
    yield 1
    yield 2
print(gen().sorted())

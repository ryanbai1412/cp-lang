// expect: R0401
// rule: R09.4
// `with checks(name: on)` takes every check name R09.3 lists; in a build that runs every check,
// `: on` changes nothing, so the assert still traps.
fn half(n: Int) -> Int:
    with checks(asserts: on, unwrap: on, div: on, reads: on):
        assert n % 2 == 0, "odd" // <- here
    n / 2
print(half(4))
print(half(3))

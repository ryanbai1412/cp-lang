// expect: ok
// rule: 09.4.1
// tool: build -o -
// has: "need 3, got "
// The judge's C++ keeps an assert under `with checks(asserts: on)`, even with every other check off (decision 390).
fn sized(n: Int) -> Int:
    with checks(asserts: on):
        assert n == 3, "need 3, got {n}"
    n
print(sized(read()))

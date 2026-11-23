// expect: exit 3
// rule: 09.4.1 09.3.7
// tool: run --release
// has: "need 3, got 2"
// `with checks(asserts: on)` keeps one assert in the judge build: a library's precondition needs no private trap (decision 390).
let a = [1, 2]
with checks(asserts: on):
    assert a.len == 3, "need 3, got {a.len}"
print(a.len)

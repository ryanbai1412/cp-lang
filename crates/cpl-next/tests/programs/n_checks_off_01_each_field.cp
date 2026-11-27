// expect: ok, output "0 0 3\n9223372036854775807 -9223372036854775808\n2\n0\n5"
// rule: R09.4.2 R09.4.3
// `with checks(x: off)` turns its check off in a checked build too: division by zero gives 0,
// `Int` arithmetic wraps and an `assert` is skipped. `unchecked:` turns every one off, and a
// check turned off in a block is on again after it.
let z = 0
with checks(div: off):
    print(7 / z, 7 % z, 7 / 2)
let big = 9223372036854775807
with checks(overflow: off):
    print(big, big + 1)
with checks(asserts: off):
    assert z == 1, "never checked"
    print(2)
unchecked:
    print(5 / z)
with checks(div: off, div: on):
    print(5 / (z + 1))

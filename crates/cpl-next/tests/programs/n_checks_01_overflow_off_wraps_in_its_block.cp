// expect: R0202
// `with checks(overflow: off)` makes `Int` arithmetic in its block wrap to 64 bits (R09.4.2);
// the block form ends at its block, the bare line at the end of the enclosing one, and
// `overflow: on` inside turns the trap back on.
fn id(k: Int) -> Int = k
let big = 4611686018427387904 + id(3) - 3
with checks(overflow: off):
    print(big * 4)
    print(id(9223372036854775807) + 2)
    print(id(3) ** 64)
    print(id(5) << 62)
    var s = 0
    for i in 0..3: s += big
    print(s)
    with checks(overflow: on):
        print(id(2) + 2)
fn g():
    with checks(overflow: off)
    print(id(-9223372036854775807) - 2)
g()
print(big * 2) // <- here

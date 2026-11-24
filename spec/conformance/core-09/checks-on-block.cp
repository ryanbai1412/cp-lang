// expect: R0202 (checked)
// rule: 09.4.1
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
let big = 4611686018427387904 + zero(3)
with checks(ranges: on, overflow: on, bounds: on):
    print(big * 4) // <- here

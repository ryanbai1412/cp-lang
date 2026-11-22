// expect: ok, output "0"
// rule: 09.4.3 13.1.4
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
let big = 4611686018427387904 + zero(3)
unchecked:
    print(big * 4)

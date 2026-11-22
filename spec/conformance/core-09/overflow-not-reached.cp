// expect: ok, output "4611686018427387904"
// rule: 09.3.4
fn zero(k: Int) -> Int = if k == 0: 0 else: zero(k - 1)
let big = 2305843009213693952 + zero(3)
print(big * 2)

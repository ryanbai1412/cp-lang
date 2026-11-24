// expect: ok, output "-9223372036854775808"
// rule: 09.2.2 09.3.4
// tool: run --release --checks none
fn one(k: Int) -> Int = if k == 0: 1 else: one(k - 1)
let a = 9223372036854775807
print(a + one(3))

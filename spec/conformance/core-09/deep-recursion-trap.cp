// expect: R0601 (checked)
// rule: 13.1.2
// timeout: 120
fn f(k: Int) -> Int = if k == 0: 0 else: 1 + f(k - 1) // <- here
print(f(100_000_000))

// expect: L0413
domain A = 0..100_000
domain B = 0..100_000
@memo
fn dp(a: A, b: B) -> Int = Int(a) + Int(b) // <- here
print(dp(A(0), B(0)))

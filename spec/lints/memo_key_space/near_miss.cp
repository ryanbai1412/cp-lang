domain A = 0..100
domain B = 0..100
@memo
fn dp(a: A, b: B) -> Int: Int(a) + Int(b)
print(dp(A.first!, B.first!))

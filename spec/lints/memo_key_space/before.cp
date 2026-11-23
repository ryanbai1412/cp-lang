domain A = 0..100_000
domain B = 0..100_000
@memo
fn dp(a: A, b: B) -> Int: a.ord + b.ord
print(dp(A.first!, B.first!))

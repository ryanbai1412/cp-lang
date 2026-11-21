struct P(x: Int)
fn (a: P) ^ (b: P) -> P = P(a.x ^ b.x)
print(1)

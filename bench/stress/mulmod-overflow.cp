let (a, b, m): (0..=1e18, 0..=1e18, 1..=1e18) = read()

fn mulmod(a: Int, b: Int, m: Int) -> Int = a * b % m

@alt(mulmod)
fn mulmod_slow(a: Int, b: Int, m: Int) -> Int = Int(I128(a) * I128(b) % I128(m))

print(mulmod(a, b, m))

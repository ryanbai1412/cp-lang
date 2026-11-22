// a * b mod m, with a planted overflow: `a * b` passes `Int.MAX` before the
// modulus is taken.
// class: overflow
let (a, b, m): (0..=1e18, 0..=1e18, 1..=1e18) = read()

fn mulmod(a: Int, b: Int, m: Int) -> Int = a * b % m

@brute(mulmod)
fn mulmod_slow(a: Int, b: Int, m: Int) -> Int = Int(I128(a) * I128(b) % I128(m))

print(mulmod(a, b, m))

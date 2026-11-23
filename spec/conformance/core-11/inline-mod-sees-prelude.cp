// expect: ok, output "2"
// rule: 11.3.1 11.1.3
// An inline `mod` in a program uses a prelude name without a `use`.
mod m:
    pub fn g() -> Int = gcd(4, 6)
print(m.g())

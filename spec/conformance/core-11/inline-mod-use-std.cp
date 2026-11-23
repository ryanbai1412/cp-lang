// expect: ok, output "2"
// rule: 11.1.3
// A `use` inside an inline module brings a std name into that module.
mod m:
    use std.math.gcd
    pub fn g() -> Int = gcd(4, 6)
print(m.g())

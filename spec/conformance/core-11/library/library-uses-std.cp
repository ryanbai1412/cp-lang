// expect: ok, output "2"
// rule: 11.3.1
// A library module names what it needs with `use`; `kit.with_use` imports `gcd`.
use kit.with_use.f
print(f())

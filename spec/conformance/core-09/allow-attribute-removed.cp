// expect: E0503
// rule: 09.4.4
// `with lint(name: allow)` is the one way to silence a lint; `@allow(name)` is a removed spelling.
@allow(float_equality) // <- here
fn half(x: F64) -> Bool = x == 0.5
print(yes(half(0.5)))

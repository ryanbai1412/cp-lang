// expect: R0201 (checked)
// rule: 04.4.24
// An `Int` argument outside a range-typed parameter traps at the call in a checked build, so the body may rely on the range.
fn half(x: 0..=100) -> Int = x / 2
fn twice_half(m: Int) -> Int = half(m * 2) // <- here
print(twice_half(100))

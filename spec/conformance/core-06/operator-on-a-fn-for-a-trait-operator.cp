// expect: E0505
// rule: 06.4.12
// engines: next
// `@operator` on a fn is only for an operator no trait gives (`and`, `or`); `+` is `Add`'s, so a fn for it would be a second path.
@operator("+") // <- here
fn plus(a: Int, b: Int) -> Int = a + b
print(plus(1, 2))

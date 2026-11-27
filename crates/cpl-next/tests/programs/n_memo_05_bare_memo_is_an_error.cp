// expect: E0311
// rule: R06.4.1
// A memo fn spells its table, `@memo(array)` or `@memo(map)`; it is never picked for it.
@memo // <- here
fn f(n: 0..=5) -> Int = n
print(f(2))

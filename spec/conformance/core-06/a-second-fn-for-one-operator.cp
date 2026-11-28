// expect: E0702
// rule: 06.4.12
// engines: next
// One operator has one fn: std's `logical_and` already gives `and`.
@operator("and") // <- here
fn both(a: Bool, b: Bool) -> Bool = a and b
print(yes(both(true, true)))

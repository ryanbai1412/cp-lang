// expect: E0505
// rule: 01.10.1
// engines: next
// has: "`@free_form` sits on a trait's method with `self`"
// Only a receiver's method has a plain spelling of its call; a plain fn already is one.
@free_form // <- here
fn twice(x: Int) -> Int = 2 * x
print(twice(3))

// expect: E0324
// `type_of` names a fn item's own type; a `let` holding a closure has none, so the error says to name a `fn` instead.
let sq = (x: Int) => x * x
fn twice(f: type_of(sq), x: Int) -> Int = f(f(x)) // <- here
print(twice(sq, 3))

// expect: E0606
// rule: 08.2.2
fn twice<T>(x: T) -> T = x + x // <- here

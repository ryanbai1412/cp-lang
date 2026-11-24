// expect: E0302
// rule: 08.2.5
struct Tagged<T, F = Int>:
    v: T
    pub fn of(v: T) -> Tagged<T, F> = Tagged<T, F>(v)
let u: Tagged<Str, Bool> = Tagged<Str>.of("a") // <- here

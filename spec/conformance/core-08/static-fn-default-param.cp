// expect: ok, output "a"
// rule: 08.2.5
struct Tagged<T, F = Int>:
    v: T
    pub fn of(v: T) -> Tagged<T, F> = Tagged<T, F>(v)
let t = Tagged<Str>.of("a")
let u: Tagged<Str, Int> = t
print(u.v)

// expect: ok, output "YES"
// rule: 06.4.3
// `gen<T>()` in a generic fn draws whatever `T` binds at the call, written or
// taken from where the value goes: a built-in, a range, a tuple of ranges, or a
// struct with its own `Random`.
struct P(a: 1..=9)
impl Random for P:
    fn gen() -> P = P(gen<1..=9>())
fn mk<T: Random>() -> T = gen<T>()
let b = mk<Bool>()
let c: Bool = mk()
let r: 1..=9 = mk()
let t: (1..=2, 3..=4) = mk()
let p: P = mk()
let s = mk<3..=3>()
print(yes((b or not b) and (c or not c) and 1 <= r <= 9 and t.0 <= 2 and t.1 >= 3 and p.a >= 1 and s == 3))

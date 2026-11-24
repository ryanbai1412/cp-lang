// expect: ok, output "YES"
// `gen_fields<T>()` draws each field with its own type's `gen()`, so a
// struct's whole `Random` is one line.
use std.derive.gen_fields
struct Q(c: 0..=1)
impl Random for Q:
    fn gen() -> Q = Q(0)
struct P(a: 1..=9, b: 5..=6, q: Q)
impl Random for P:
    fn gen() -> P = gen_fields<P>()
var ok = true
for _ in 0..100:
    let p = gen<P>()
    ok = ok and p.a >= 1 and p.a <= 9 and p.b >= 5 and p.b <= 6 and p.q.c == 0
print(yes(ok))

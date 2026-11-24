// expect: ok, output "3 2 5"
// rule: 02.1.3
// A struct's static fn named without a call is a fn value like any `fn` item (decision 472).
struct P:
    x: Int
    fn mk(v: Int) -> P = P(v)
    fn add(a: P, b: P) -> P = P(a.x + b.x)
let f = P.mk
print(f(3).x, [1, 2].map(P.mk)[1].x, [2, 3].map(P.mk).fold(P.add)!.x)

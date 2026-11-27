// expect: ok, output "3 4\n7\n1 2"
// rule: R08.2.5
// Type arguments left off the end of a struct's name take its defaults, which may name the
// parameters before them, in a type and on a constructor or static call alike.
struct Pair<A, B = A>:
    a: A
    b: B
    pub fn of(x: A, y: B) -> Pair<A, B> = Pair<A, B>(x, y)
struct Tagged<T, N = Int>:
    v: T
    n: N
fn sum(p: Pair<Int>) -> Int = p.a + p.b
fn show(p: Pair<Int>) -> Pair<Int, Int> = p
let p: Pair<Int> = Pair(3, 4)
let q = show(p)
print(q.a, q.b)
let t: Tagged<Bool> = Tagged(true, sum(p))
print(t.n)
let r: Pair<Int, Int> = Pair<Int>(1, 2)
let s: Pair<Int, Int> = Pair<Int>.of(r.b, r.b)
print(r.a, s.b)

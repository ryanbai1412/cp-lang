// expect: ok, output "3 x 8 13 7"
// rule: R08.2.5
// Written type arguments: a generic fn named with them is a fn value, a method takes its own.
fn id<T>(x: T) -> T = x
fn first<A, B>(a: A, b: B) -> A = a
struct Acc<T>:
    var base: T
    fn plus<U>(self, v: U, f: (T, U) -> T) -> T = f(base, v)
fn twice(f: Int -> Int, x: Int) -> Int = f(f(x))
let g = id<Int>
let h = id<Str>
let k = first<Int, Bool>
let a = Acc(5)
print(g(3), h("x"), twice(x => g(x) * 2, 2), a.plus<Int>(4, (b, v) => b + v * 2), k(7, true))

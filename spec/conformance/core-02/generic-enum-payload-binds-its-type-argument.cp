// expect: ok, output "42"
// rule: 02.7.1
enum Boxed<T> = One(T) | Empty
let b = Boxed.One(42)
match b:
    One(k) => print(k)
    Empty => print(0)

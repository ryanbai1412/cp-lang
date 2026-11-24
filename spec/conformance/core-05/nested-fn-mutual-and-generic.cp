// expect: ok, output "1"
// rule: 05.10.6 06.1.7
fn f() -> Bool:
    fn even(n: Int) -> Bool = if n == 0: true else: odd(n - 1)
    fn odd(n: Int) -> Bool = if n == 0: false else: even(n - 1)
    fn id<T>(x: T) -> T = x
    id(even(10))
print(if f(): 1 else: 0)

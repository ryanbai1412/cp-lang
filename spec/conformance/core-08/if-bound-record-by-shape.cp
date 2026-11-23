// expect: ok, output "5 0"
// rule: 08.1.7 08.2.9
// A record whose fields meet every bodiless member of a trait satisfies it in `if A: Trait` too.
trait Named:
    @get fn name(self) -> Int
fn f<A>(a: A) -> Int:
    if A: Named:
        return a.name
    0
print(f((name: 5, x: 1)), f(3))

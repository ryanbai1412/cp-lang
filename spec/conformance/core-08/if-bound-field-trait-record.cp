// expect: ok, output "5 5"
// rule: 08.1.7 08.2.9
// `if A: Trait` and a bound `A: Trait` agree: a record meets a field trait in both.
trait HasW: w: Int
fn f<A>(a: A) -> Int:
    if A: HasW:
        return a.w
    0
fn g<A: HasW>(a: A) -> Int = a.w
print(f((w: 5, z: 1)), g((w: 5, z: 1)))

// expect: ok, output "6"
// The shape comes from `new`'s signature, not the type's name: a program's own family gets the same rule.
struct Tally<D, T>:
    n: Int
    fn new(a: [D]T) -> Tally<D, T> = Tally<D, T>(1)
struct Tally<T>:
    n: Int
    fn new(a: List<T>) -> Tally<T> = Tally<T>(a.len * 2)
let t: Tally<Int> = Tally([1, 2, 3])
print(t.n)

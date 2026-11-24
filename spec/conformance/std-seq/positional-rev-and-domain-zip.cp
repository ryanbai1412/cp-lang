// expect: ok, output "5 5 5 5\n0 1 2\n12 9\n3"
// rule: 07.3.7 07.6.4
// `zip`'s later argument is `Positional`: an `Indexed`, the `.rev` of one, or a domain, each
// read by walk position, while a generic fn bounded `Positional<U>` reads `nth` from the end
// of a `.rev`.
fn last_of<B: Positional<U>, U>(b: B) -> (Int, U) = (b.len, b.nth(b.len - 1))
domain R = 0..3
let a = [1, 2, 3, 4]
print([x + y for (x, y) in zip(a, a.rev)])
print([r for (_, r) in zip([7, 8, 9], R)])
let (n, y) = last_of([12, 9].rev)
print(y, last_of([12, 9]).1)
print(last_of(R).0)

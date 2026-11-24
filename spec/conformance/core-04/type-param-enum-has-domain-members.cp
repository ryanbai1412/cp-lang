// expect: ok, output "b 3 R 3 5 R 2"
// rule: 04.2.13
// A type param filled by an enum, a domain or `Char` has the same domain members: `try_at`, `all`, `wrap`.
enum Shape: L | R | X
domain D = 0..5
fn pick<A>(c: Int) -> A = A.try_at(c)!
fn n<A>() -> Int = A.all.len
fn spin<A>(k: Int) -> A = A.wrap(k)
print(pick<Char>(98), pick<D>(3), pick<Shape>(1), n<Shape>(), n<D>(), spin<Shape>(4), spin<D>(7))

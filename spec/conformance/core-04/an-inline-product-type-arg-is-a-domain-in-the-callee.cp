// expect: ok, output "6 6 6\n3"
// rule: 04.2.20
// An inline product `(V, W)` given as a generic fn's domain is that fn's
// domain with every position, before anything else names it: a
// comprehension, a `sum` and a list walk all of them.
domain V = [2]
domain W = [3]
fn positions<D>() -> Int = count(true for x in D)
fn total<D>() -> Int = sum(1 for x in D)
fn cells<D>() -> Int = [x for x in D].len
fn tally<D>(p: D -> Bool) -> Int = count(p(x) for x in D)
print(positions<(V, W)>(), total<(V, W)>(), cells<(V, W)>())
print(tally<(V, V)>((u, v) => u <= v))

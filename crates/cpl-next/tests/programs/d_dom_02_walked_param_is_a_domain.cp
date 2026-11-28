// expect: ok, output "3\n0 1 2\n4"
// rule: R04.2
// A type parameter that a body walks as a value (`for v in V`, `[v for v in V]`) is a domain
// parameter, as one the body counts or indexes by is: it needs no written `Domain`.
fn size<V>() -> Int:
    var n = 0
    for v in V:
        n += 1
    n
fn ords<V>() -> List<Int> = [v.ord for v in V]
domain V = 0..3
domain W = 2..6
print(size<V>())
print(ords<V>())
print(size<W>())

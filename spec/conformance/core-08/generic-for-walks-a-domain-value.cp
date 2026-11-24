// expect: ok, output "4 4 2\n3"
// rule: 08.5.3
// A domain passed for `A: Seq<T>` is the walk of its positions, so a generic `for` goes over it
// as it goes over a list or a reversed domain.
fn walk<A: Seq<T>, T>(a: A) -> Int:
    var n = 0
    for x in a: n += 1
    n
fn last<A: Seq<T>, T>(a: A) -> T:
    var y = a.first!
    for x in a: y = x
    y
domain R = 0..4
print(walk(R), walk(R.rev), walk([1, 2].rev))
print(last(R).ord)

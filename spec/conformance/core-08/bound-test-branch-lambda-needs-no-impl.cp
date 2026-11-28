// expect: ok, output "1 0 2 -1"
// rule: 08.2.9
// engines: next
// A lambda written in the branch an `if T: Trait:` test guards is made only where the test
// holds, so an instance that fails it needs nothing the lambda's body calls. `Tag` has no `Eq`.
struct Tag:
    f: Int -> Int
fn count<T>(x: T) -> Int:
    var keep: List<T -> Bool> = []
    if T: Eq:
        keep.push(a => a == x)
    keep.len
fn hits<T>(xs: List<T>, x: T) -> Int:
    if T: Eq:
        let same = (a: T) => a == x
        var n = 0
        for a in xs:
            if same(a):
                n += 1
        return n
    -1
print(count(3), count(Tag(i => i)), hits([1, 2, 1], 1), hits([Tag(i => i)], Tag(i => i)))

// expect: ok, output "3 ab"
// rule: 07.8.1
// A generic `S: Seq<T>` converts to `List<T>` by its blanket impl, though `Seq<Char>` also converts to `Str`.
fn n<S: Seq<T>, T>(s: S) -> Int:
    let xs: List<T> = List(s)
    xs.len
fn w<S: Seq<Char>>(s: S) -> Str = Str(s)
print(n(1..4), w(['a', 'b']))

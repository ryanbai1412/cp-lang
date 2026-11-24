// expect: ok, output "3 2 5 | 0 0"
// rule: std.seq.Sized
// `List`, `Str` and `Range` meet `Sized`, and `List` and `Str` meet `Keyed<Int, _>`, through std's own `impl Sized for List<T>` and the like, not by name.
fn n<S: Sized>(s: S) -> Int = s.len
fn first_key<K, T, S: Keyed<K, T>>(s: S) -> ?K:
    for (k, _) in s.items: return k
    return None
print(n([1, 2, 3]), n("ab"), n(0..5), "|", first_key([7, 8]) ?? -1, first_key("xy") ?? -1)

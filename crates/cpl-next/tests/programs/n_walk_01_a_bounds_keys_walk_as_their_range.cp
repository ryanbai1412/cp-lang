// expect: ok, output "0 1 2\n3 1 2\n1 2 3\n7 8 9"
// rule: R07.1.1 R08.1.1
// `for k in s.keys`, with `s` of a type parameter bound by `Indexed`, walks the `Range<D>` that
// `keys` returns, the way a value of that type walks; std's `sort` is one such walk.
use std.seq.{Indexed}
fn keys_of<S: Indexed<D, T>, D: Domain, T>(s: S) -> List<D>:
    var ks: List<D> = []
    for k in s.keys: ks.push(k)
    ks
fn firsts<S: Indexed<D, T>, D: Domain, T>(s: S) -> List<T>:
    var xs: List<T> = []
    for k in s.keys: xs.push(s[k])
    xs
var a = [3, 1, 2]
print(keys_of(a))
print(firsts(a))
a.sort()
print(a)
var b = [9, 8, 7]
b.sort()
print(b)

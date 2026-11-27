// expect: ok, output "5 3 ab"
// rule: R06.4.12
// `?T`, `[a, b]` and `"s"` are std's `@optional`, `@list` and `@str` structs, not whatever is
// named `Optional`, `List` or `Str` where they are written: the program's own structs of those
// names change none of them, and `+` on strings is `Str`'s `Add` impl.
struct Optional:
    k: Int
struct List:
    k: Int
struct Str:
    k: Int
let a: ?Int = Some(5)
let xs = [1, 2, 3]
let s = "a" + "b"
if let Some(v) = a:
    print(v, xs.len, s)

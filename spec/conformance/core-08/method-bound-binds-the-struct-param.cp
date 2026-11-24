// expect: ok, output "0 3 3 b"
// rule: 08.3
// A bound `S: Indexed<K, T>` binds the names it uses from what `S` is, in a free fn and in a
// static method alike, the struct's own `T` included.
fn first_key<S: Indexed<E, R>, E, R>(xs: S) -> ?E = xs.keys.first
struct Box<T: Ord>:
    n: Int
    src: List<T>
    fn new<S: Indexed<K, T>, K>(s: S) -> Box<T>:
        let src = [s.nth(i) for i in 0..s.len]
        Box(src.len, src)
    fn of<S: Indexed<K, T>, K>(s: S) -> Box<T> = Box.new(s)
let b = Box("abc")
let c = Box.of([5, 6, 7])
print(first_key([5, 6])!, b.n, c.n, b.src[1])

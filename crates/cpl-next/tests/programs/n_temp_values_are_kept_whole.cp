// expect: ok, output "3\n1\n5\n321\n3\n6\n3"
// rule: R07.5.10
// A temporary dies at the end of its statement, but only a view cut from it dies with it: an
// element read out of it is a copy, a literal is the program's own value, a `for` header's
// temporary lives for the loop, and a fresh value passed to a holding parameter moves in.
struct Box<T>:
    v: T
    pub fn of(v: T) -> Box<T> = Box<T>(v)
fn run():
    let a = [3, 1, 2].sorted()[2]
    var b = [3, 1, 2].sorted()[0]
    print(a)
    print(b)
    var n = 0
    for x in [3, 1, 2, 4].sorted()[1..3]:
        n += x
    print(n + b - 1)
    let r = [1, 2, 3].rev
    var o = 0
    for x in r:
        o = o * 10 + x
    print(o)
    let s = "abc".rev
    var m = 0
    for c in s:
        m += 1
    print(m)
    let t = Box.of([1, 2, 3].sorted())
    print(t.v.sum())
    print(t.v.len)
run()

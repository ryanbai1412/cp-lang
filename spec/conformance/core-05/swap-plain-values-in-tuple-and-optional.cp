// expect: ok, output "6 5 15 10 4 1"
// rule: 05.8.2
struct P(a: Int, b: Int)

fn go(n: Int):
    var t = (n, n + 1)
    swap(mut t.0, mut t.1)
    var o: ?Int = Some(n * 2)
    var q: ?Int = Some(n * 3)
    swap(mut o!, mut q!)
    var u = (P(a: 3, b: 4), n)
    var w = P(a: 1, b: 2)
    swap(mut u.0, mut w)
    print(t.0, t.1, o!, q!, w.b, u.0.a)

go(5)

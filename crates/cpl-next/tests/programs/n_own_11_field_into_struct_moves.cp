// expect: ok, output "1 2"
// rule: R05.2.2
// A field read into a new struct moves into it (R2), so returning the struct does not borrow the
// local the field was read from.
struct P(a: List<Int>, b: Int)
struct Q(a: List<Int>)
fn mk() -> P = P([1, 2], 3)
fn q() -> Q:
    let t = mk()
    Q(t.a)
print(q().a)

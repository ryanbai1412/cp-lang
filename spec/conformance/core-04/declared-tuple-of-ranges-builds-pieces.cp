// expect: ok, output "2 4"
// rule: 04.8.5
// A declared tuple of `Range<V>` is a place a `Range<V>` is expected, so each cut-ended literal in it builds the piece.
domain X = 1..=10
fn size(r: Range<X>) -> Int = r.count()
let a = X(2)..X(5)
let b = X(4)..X(8)
let (left, right): (Range<X>, Range<X>) = (a.l..b.l, b.l..b.r)
print(size(left), size(right))

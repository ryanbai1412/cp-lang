// expect: ok, output "1 2"
// rule: 06.1.5
// The value `()` has the type `()`, so a generic parameter given `()` takes the literal `()`,
// passed or as its default (`tag: P = ()` in `MonotoneHull.add`).
fn f<P>(tag: P = ()) -> Int = 1
fn g<P>(tag: P) -> Int = 2
print(f<()>(), g<()>(()))

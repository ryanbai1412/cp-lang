// expect: ok, output "1 1"
// rule: 09.1.3
// A top-level fn without `uses` passes its own lexical `io` on, not its caller's.
fn inner(a: Int) uses io = print(a, a)
fn outer(a: Int) = inner(a)
with io(sep: ","):
    outer(1)

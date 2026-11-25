// expect: ok, output "6"
// rule: R05.10.1: top-level names are not captures
// A top-level `var` read by a fn before its declaration line runs (the fn is called after): legal, sees the value.
fn get() -> Int = base * 2
var base = 3
print(get())

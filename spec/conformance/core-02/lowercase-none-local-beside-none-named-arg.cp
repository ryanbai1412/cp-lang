// expect: ok, output "1 5"
// A local named `none` is an ordinary name (c973f0d5f): `print`'s `none:` argument still takes it, and `None` stays the literal.
let none = 5
print([Some(1), None], none: none)

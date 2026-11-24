// expect: E0203
// rule: 08.1.6
// A tuple has only its positions and the fns that take it; `foo` is neither, so it is an error, not an unknown type.
let t = (1, 2)
print(t.foo) // <- here

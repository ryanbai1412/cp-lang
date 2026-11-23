// expect: ok, output "2 2 2"
// `present()` unwraps any `?T`, including the `?` pieces `minimal` yields per right end, as in the spec's own `a.minimal(Window(+), ok:).present()` example (rebuild round 34 F4).
let a: [4]Int = [1, 2, 3, 4]
print([r.len for r in a.minimal(Window(+), ok: w => (w.all ?? 0) >= 5).present()])

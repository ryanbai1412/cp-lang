// expect: E0204
// rule: diagnostics
// lacks: "slice:"
// `!`, `??`, `if let` and let-else on a value whose type is built from an error say nothing more.
fn f() -> Int:
    let xs: List<Nope> = [] // <- here
    let a = xs!
    let b = xs ?? xs
    if let y = xs: print(1)
    let z = xs else: return 0
    1
print(f())

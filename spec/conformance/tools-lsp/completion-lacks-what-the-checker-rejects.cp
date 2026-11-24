// expect: ok
// rule: lsp.2
// tool: lsp completion g.|
// has: "label":"sum"
// lacks: "label":"rev"
// Completion offers only what the checker accepts: a generator is not `Bidir`, so `.rev` is not offered on it (decision 513).
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i
let g = evens(7)
g.

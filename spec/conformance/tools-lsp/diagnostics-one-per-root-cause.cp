// expect: ok
// rule: lsp.1
// tool: lsp diagnostics
// has: errors/E0201
// lacks: errors/E0302
// One root cause gives one diagnostic: the unknown name, with no type errors where `x` is used.
let x = missing_name
print(x + 1, x * 2)

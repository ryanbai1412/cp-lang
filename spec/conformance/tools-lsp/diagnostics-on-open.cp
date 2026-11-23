// expect: ok
// rule: lsp.1
// tool: lsp diagnostics
// has: errors/E0302
// has: "severity":1
let x: Int = "a"
print(x)
